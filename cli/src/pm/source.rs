use anyhow::{bail, Result};
use clap::ArgMatches;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum PackageSource {
    Git {
        user: String,
        repo: String,
        branch: Option<String>,
        tag: Option<String>,
    },
    Path(PathBuf),
    Registry,
}

impl PackageSource {
    pub fn from_arg_matches(matches: &ArgMatches) -> Result<PackageSource> {
        if let Some(git) = matches.get_one::<String>("git") {
            PackageSource::git_from_string(git.clone())
        } else if let Some(path) = matches.get_one::<PathBuf>("path") {
            Ok(PackageSource::Path(path.clone()))
        } else {
            Ok(PackageSource::Registry)
        }
    }

    pub fn git_from_string(git: String) -> Result<PackageSource> {
        let mut user = String::new();
        let mut repo = String::new();
        let mut branch = None;
        let mut tag = None;

        let stripped = git
            .trim_start_matches("https://")
            .trim_start_matches("github.com/")
            .trim();

        let parts: Vec<&str> = stripped.split(|c| c == '#' || c == '@').collect();

        match parts.len() {
            1 => {
                let mut segments = parts[0].split('/');
                user = segments.next().unwrap_or_default().to_string();
                repo = segments.next().unwrap_or_default().to_string();
            }
            2 => {
                let mut segments = parts[0].split('/');
                user = segments.next().unwrap_or_default().to_string();
                repo = segments.next().unwrap_or_default().to_string();

                if stripped.contains('#') {
                    branch = Some(parts[1].to_string());
                } else {
                    tag = Some(parts[1].to_string());
                }
            }
            _ => bail!("Invalid GitHub repository format: {}", git),
        }

        if user.is_empty() || repo.is_empty() {
            bail!("Invalid GitHub repository format: {}", git);
        }

        Ok(PackageSource::Git {
            user,
            repo,
            branch,
            tag,
        })
    }
}
