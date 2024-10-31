use crate::{config_file::RoanConfig, fs::walk_for_file, shell::Shell};
use anstream::ColorChoice;
use anyhow::{anyhow, Context, Result};
use clap::ArgMatches;
use roan_engine::path::normalize_path;
use std::{fs::read_to_string, path::PathBuf, time::Instant};

#[derive(Debug)]
pub struct GlobalContext {
    pub verbose: bool,
    pub cwd: PathBuf,
    pub config: Option<RoanConfig>,
    pub start: Instant,
    pub shell: Shell,
}

impl GlobalContext {
    pub fn default(color_choice: ColorChoice) -> Result<Self> {
        Ok(Self {
            verbose: false,
            cwd: std::env::current_dir().context("Failed to get current directory")?,
            config: None,
            start: Instant::now(),
            shell: Shell::new(color_choice),
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

        let path = normalize_path(file, self.cwd.clone())?;
        if !path.exists() {
            return Err(anyhow!("Main file does not exist: {}", path.display()));
        }

        Ok(path)
    }

    pub fn project_type(&self) -> Result<&str> {
        Ok(self.get_config()?.project.r#type.as_ref().unwrap())
    }

    pub fn build_dir(&self) -> Result<PathBuf> {
        Ok(self.cwd.join("build"))
    }

    pub fn deps_dir(&self) -> Result<PathBuf> {
        Ok(self.build_dir()?.join("deps"))
    }
}
