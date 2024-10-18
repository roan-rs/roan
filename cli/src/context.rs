use crate::{config_file::RoanConfig, fs::walk_for_file};
use anyhow::{Context, Result};
use std::{fs::read_to_string, path::PathBuf};

#[derive(Debug)]
pub struct GlobalContext {
    pub verbose: bool,
    pub cwd: PathBuf,
    pub config: Option<RoanConfig>,
}

impl GlobalContext {
    pub fn default() -> Result<Self> {
        Ok(Self {
            verbose: false,
            cwd: std::env::current_dir().context("Failed to get current directory")?,
            config: None,
        })
    }

    pub fn load_config(&mut self) -> Result<RoanConfig> {
        let path = walk_for_file(self.cwd.clone(), "roan.toml").context(
            "Failed to find roan.toml. Make sure you are running the command inside project root or in a subdirectory",
        )?;

        let content = read_to_string(&path).context("Failed to read roan.toml")?;
        let config: RoanConfig = toml::from_str(&content)?;

        self.config = Some(config.clone());

        Ok(config)
    }
}
