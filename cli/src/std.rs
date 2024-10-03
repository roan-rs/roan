use std::fs::create_dir;
use std::path::PathBuf;
use anyhow::Result;
use dirs::home_dir;
use log::debug;

pub fn prepare_home_dir() -> Result<PathBuf> {
    let home_dir = home_dir().expect("Could not find home directory");
    let roan_dir = home_dir.join(".roan");

    if !roan_dir.exists() {
        create_dir(&roan_dir)?;
        debug!("Created roan directory at {:?}", roan_dir);
    }

    Ok(roan_dir)
}

