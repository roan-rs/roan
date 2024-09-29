use std::path::{Path, PathBuf};
use crate::module::loader::ModuleLoader;
use anyhow::Result;

/// A module loader that loads modules from the filesystem.
#[derive(Clone, Debug)]
pub struct FsModuleLoader {
    root: PathBuf,
}

impl FsModuleLoader {
    /// Create a new [`FsModuleLoader`] from a root path.
    ///
    /// # Errors
    /// An error happens if the root path cannot be canonicalized (e.g. does
    /// not exist).
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        let root = root.canonicalize().map_err(|e| {
            anyhow::anyhow!("Failed to canonicalize root path: {}", e)
        })?;

        Ok(Self { root })
    }
}

impl ModuleLoader for FsModuleLoader {
    fn load(&self) {
        println!("Loading module from filesystem");
    }
}