use crate::{context::Context, module::Module};
use log::debug;
use std::{
    fmt::Debug,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub mod basic;

/// Trait that defines the interface for a module loader.
pub trait ModuleLoader: Debug {
    /// Load a module from a given source.
    fn load(
        &self,
        referrer: &Module,
        spec: &str,
        ctx: &Context,
    ) -> anyhow::Result<Arc<Mutex<Module>>>;

    /// Insert a module into the loader's cache if loader handles caching.
    ///
    /// This function is a no-op for loaders that do not cache modules.
    ///
    /// # Arguments
    /// - `name` - The name of the module to insert into the cache.
    /// - `module` - The module to insert into the cache.
    fn insert(&self, name: String, module: Arc<Mutex<Module>>) {}

    /// Get a module from the cache if the loader caches modules.
    ///
    /// This function returns `None` for loaders that do not cache modules.
    ///
    /// # Arguments
    /// - `name` - The name of the module to get from the cache.
    fn get(&self, name: &str) -> Option<Arc<Mutex<Module>>> {
        None
    }

    /// Resolves the path of a referenced module based on the referrer module's path and the provided specification.
    ///
    /// # Arguments
    ///
    /// * `referrer` - A reference to the `Module` that provides the context for resolving the path.
    /// * `spec` - A string slice that represents the specification of the path to resolve.
    ///
    /// # Returns
    ///
    /// A `Result<PathBuf>`, where the `Ok` variant contains the resolved path, and the `Err` variant
    /// contains an error if the operation fails (e.g., if the `referrer` path has no parent).
    ///
    /// # Panics
    ///
    /// This function will panic if the `referrer` module's path has no parent directory.
    fn resolve_referrer(&self, referrer: &Module, spec: &str) -> anyhow::Result<PathBuf> {
        debug!("Resolving referrer: {:?}, spec: {}", referrer.path(), spec);
        let referrer_path = referrer
            .path()
            .map_or_else(|| PathBuf::new(), |p| p.to_path_buf());
        let dir = referrer_path.parent().expect("Module path has no parent");

        let spec = if cfg!(windows) {
            spec.replace("/", "\\")
        } else {
            spec.to_string()
        };
        let str_path = remove_surrounding_quotes(&spec);

        let spec_path = PathBuf::from(str_path);

        let path = if spec_path.is_absolute() {
            spec_path
        } else {
            dir.join(spec_path)
        };
        debug!("Resolved path: {:?}", path);

        Ok(path)
    }
}

/// Removes surrounding double quotes from a string slice if present.
pub fn remove_surrounding_quotes(s: &str) -> &str {
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        &s[1..s.len() - 1]
    } else {
        s
    }
}
