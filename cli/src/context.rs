use crate::{
    config_file::{Dependency, RoanConfig},
    fs::walk_for_file,
    pm::entry::InstallEntry,
    shell::Shell,
};
use anstream::ColorChoice;
use anyhow::{anyhow, bail, Context, Result};
use colored::Colorize;
use flate2::read::GzDecoder;
use http_body_util::BodyExt;
use octocrab::{
    models::repos::Object,
    params::repos::{Commitish, Reference},
    Octocrab,
};
use roan_engine::path::{canonicalize_path, normalize_path, normalize_without_canonicalize};
use std::{
    collections::HashMap,
    fs::{read_to_string, File},
    io::{BufReader, Cursor, Read, Write},
    path::PathBuf,
    sync::Arc,
    time::Instant,
};
use tracing::debug;

#[derive(Debug)]
pub struct GlobalContext {
    pub verbose: bool,
    pub cwd: PathBuf,
    pub config: Option<RoanConfig>,
    pub start: Instant,
    pub shell: Shell,
    pub octocrab: Arc<Octocrab>,
}

impl GlobalContext {
    pub fn default(color_choice: ColorChoice) -> Result<Self> {
        Ok(Self {
            verbose: false,
            cwd: std::env::current_dir().context("Failed to get current directory")?,
            config: None,
            start: Instant::now(),
            shell: Shell::new(color_choice),
            octocrab: octocrab::instance(),
        })
    }

    pub fn from_cwd(cwd: PathBuf, color_choice: ColorChoice) -> Result<Self> {
        Ok(Self {
            verbose: false,
            cwd,
            config: None,
            start: Instant::now(),
            shell: Shell::new(color_choice),
            octocrab: octocrab::instance(),
        })
    }

    pub fn load_config(&mut self) -> Result<RoanConfig> {
        let path = walk_for_file(self.cwd.clone(), "roan.toml").context(
            "Failed to find roan.toml. Make sure you are running the command inside project root or in a subdirectory",
        )?;

        let content = read_to_string(&path).context("Failed to read roan.toml")?;
        let config: RoanConfig = toml::from_str(&content)?;

        self.config = Some(config.clone());

        if config.project.r#type.is_none() {
            return Err(anyhow!(
                "Project type is not specified in [project] in roan.toml. Available types: 'lib', 'bin'"
            ));
        }

        let r#type = self.project_type()?;
        if r#type != "lib" && r#type != "bin" {
            return Err(anyhow!(
                "Invalid project type in [project] in roan.toml. Available types: 'lib', 'bin'"
            ));
        }

        Ok(config)
    }

    pub fn get_config(&self) -> Result<&RoanConfig> {
        self.config
            .as_ref()
            .ok_or_else(|| anyhow!("Config is not loaded"))
    }

    pub fn get_config_mut(&mut self) -> Result<&mut RoanConfig> {
        self.config
            .as_mut()
            .ok_or_else(|| anyhow!("Config is not loaded"))
    }

    pub fn get_main_file(&self) -> Result<PathBuf> {
        let config = self.get_config()?.clone();

        // We unwrap here because we have already checked that project type is specified in config
        let file: PathBuf = match self.project_type()? {
            "lib" => config
                .project
                .lib
                .clone()
                .unwrap_or_else(|| "src/lib.roan".into()),
            "bin" => config
                .project
                .bin
                .clone()
                .unwrap_or_else(|| "src/main.roan".into()),
            _ => unreachable!(),
        };

        let path = normalize_without_canonicalize(file, self.cwd.clone());
        if !path.exists() {
            return Err(anyhow!("Main file does not exist: {}", path.display()));
        }

        Ok(canonicalize_path(path)?)
    }

    // Parent directory of the main file
    pub fn get_main_dir(&self) -> Result<PathBuf> {
        Ok(self.get_main_file()?.parent().unwrap().to_path_buf())
    }

    pub fn project_type(&self) -> Result<&str> {
        Ok(self.get_config()?.project.r#type.as_ref().unwrap())
    }

    pub fn assert_type(&self, r#type: &str) -> Result<()> {
        if self.project_type()? != r#type {
            bail!(
                "Expected project {} to be of type {}",
                self.cwd.display().to_string().dimmed(),
                r#type.bright_magenta()
            );
        }

        Ok(())
    }

    pub fn build_dir(&self) -> Result<PathBuf> {
        Ok(self.cwd.join("build"))
    }

    pub fn deps_dir(&self) -> Result<PathBuf> {
        Ok(self.build_dir()?.join("deps"))
    }

    pub fn cache_dir(&self) -> Result<PathBuf> {
        Ok(dirs::cache_dir().unwrap().join("roan"))
    }

    pub async fn install(&mut self, entry: InstallEntry) -> Result<()> {
        let repo = self.octocrab.repos(entry.user(), entry.repo());
        let repository = repo.get().await?;

        let git_ref = match (entry.branch(), entry.tag()) {
            (Some(branch), None) => &Reference::Branch(branch.to_string()),
            (None, Some(tag)) => &Reference::Tag(tag.to_string()),
            (None, None) => {
                let branch = repository.default_branch.as_deref().unwrap_or("master");
                &Reference::Branch(branch.to_string())
            }
            _ => return Err(anyhow::anyhow!("Both branch and tag are specified")),
        };

        if let Some(url) = repository.html_url {
            self.shell.status("Resolved", &url)?;
        }

        let file_path = entry.file_name();
        let cache_dest = self.cache_dir()?.join(file_path.clone());
        debug!("Cache destination: {:?}", cache_dest);
        let final_dest = self.deps_dir()?.join(entry.repo());

        if cache_dest.exists() {
            self.shell.status("Using cache", &cache_dest.display())?;
            self.unpack_tar(cache_dest.clone(), final_dest.clone())?;

            self.update_dependencies(entry)?;

            return Ok(());
        }

        let mut tarball_stream = repo.download_tarball(git_ref.clone()).await?;

        let mut file = File::create(&cache_dest)?;

        while let Some(next) = tarball_stream.frame().await {
            let frame = next?;
            if let Some(chunk) = frame.data_ref() {
                file.write_all(chunk)?;
            }
        }

        self.shell.status("Downloaded", &cache_dest.display())?;
        self.unpack_tar(cache_dest.clone(), final_dest)?;

        self.update_dependencies(entry)?;

        Ok(())
    }

    pub fn update_dependencies(&mut self, entry: InstallEntry) -> Result<()> {
        let config = self.get_config_mut()?;

        let dep = Dependency {
            github: Some(format!("{}/{}", entry.user(), entry.repo())),
            version: entry.tag().map(|s| s.to_string()),
            branch: entry.branch().map(|s| s.to_string()),
            path: None,
        };

        if let Some(mut deps) = config.dependencies.clone() {
            deps.insert(entry.repo().into(), dep);
            config.dependencies = Some(deps)
        } else {
            config.dependencies = Some(HashMap::new());

            config
                .dependencies
                .as_mut()
                .unwrap()
                .insert(entry.repo().into(), dep);
        }

        self.config = Some(config.clone());
        self.update_config()?;

        Ok(())
    }

    pub fn update_config(&mut self) -> Result<()> {
        let config = self.get_config()?;
        let path = self.cwd.clone().join("roan.toml");

        let toml = toml::to_string(&config)?;

        std::fs::write(path, toml)?;

        Ok(())
    }

    pub fn unpack_tar(&self, path: PathBuf, to: PathBuf) -> Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let decompressed = GzDecoder::new(reader);
        let mut archive = tar::Archive::new(decompressed);

        for entry in archive.entries()? {
            let mut entry = entry?;
            let entry_path = entry
                .path()?
                .to_path_buf()
                .components()
                .skip(1)
                .collect::<PathBuf>();
            let dest_path = to.join(entry_path);

            debug!("Unpacking: {:?}", dest_path);

            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            #[cfg(target_os = "windows")]
            if entry.header().entry_type().is_symlink() {
                debug!("Skipping symlink: {:?}", entry.path()?);
                continue;
            }

            entry.unpack(dest_path)?;
        }

        Ok(())
    }
}
