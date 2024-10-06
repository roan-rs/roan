use crate::{context::Context, module::Module};
use anyhow::Result;
use log::debug;
use roan_ast::source::Source;
use std::{
    collections::HashMap,
    fmt::Debug,
    path::PathBuf,
    sync::{Arc, Mutex},
};

/// Trait that defines the interface for a module loader.
pub trait ModuleLoader: Debug {
    /// Load a module from a given source.
    fn load(&self, referrer: &Module, spec: &str, ctx: &Context) -> Result<Arc<Mutex<Module>>>;

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
    fn resolve_referrer(&self, referrer: &Module, spec: &str) -> Result<PathBuf> {
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

/// A basic implementation of the `ModuleLoader` trait that caches modules in memory.
#[derive(Debug)]
pub struct BasicModuleLoader {
    modules: Arc<Mutex<HashMap<String, Arc<Mutex<Module>>>>>,
}

impl BasicModuleLoader {
    /// Creates a new [`BasicModuleLoader`] with an empty cache of modules.
    pub fn new() -> Self {
        debug!("Creating new BasicModuleLoader");
        Self {
            modules: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Creates a new [`BasicModuleLoader`] with the specified cache of modules.
    pub fn with_modules(cache: Arc<Mutex<HashMap<String, Arc<Mutex<Module>>>>>) -> Self {
        debug!("Creating new BasicModuleLoader with provided module cache");
        Self { modules: cache }
    }

    /// Returns a clone of the module cache.
    pub fn modules(&self) -> Arc<Mutex<HashMap<String, Arc<Mutex<Module>>>>> {
        Arc::clone(&self.modules)
    }
}

impl ModuleLoader for BasicModuleLoader {
    /// Loads a module based on the specification `spec` relative to the `referrer` module.
    ///
    /// If the module is already in the cache, it returns the cached module.
    /// Otherwise, it resolves the path, loads the module, caches it, and returns it.
    fn load(&self, referrer: &Module, spec: &str, ctx: &Context) -> Result<Arc<Mutex<Module>>> {
        debug!("Loading module: {}", spec);

        {
            // Attempt to retrieve the module from the cache.
            let cache_key = remove_surrounding_quotes(spec).to_string();
            let cache = self.modules.lock().expect("Failed to lock module cache");
            if let Some(module) = cache.get(&cache_key) {
                debug!("Module found in cache: {}", cache_key);
                return Ok(Arc::clone(module));
            }
        }

        // Use the resolved path as the cache key to prevent duplicates.
        let resolved_path = self.resolve_referrer(referrer, spec)?;
        let cache_key = resolved_path.to_string_lossy().to_string();

        // Module not in cache; proceed to load.
        debug!(
            "Module not found in cache. Loading from path: {:?}",
            resolved_path
        );
        let source = Source::from_path(resolved_path)?;
        let module = Module::new(source); // Now returns Arc<Mutex<Module>>

        // Insert the newly loaded module into the cache.
        {
            let mut cache = self.modules.lock().expect("Failed to lock module cache");
            cache.insert(cache_key.clone(), Arc::clone(&module));
            debug!("Module loaded and cached: {}", cache_key);
        }

        Ok(module)
    }

    /// Inserts a module into the loader's cache.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the module to insert into the cache.
    /// * `module` - The module to insert into the cache.
    fn insert(&self, name: String, module: Arc<Mutex<Module>>) {
        debug!("Inserting module into cache: {}", name);
        let mut cache = self.modules.lock().unwrap();
        cache.insert(name, module);
    }

    /// Retrieves a module from the cache.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the module to retrieve from the cache.
    ///
    /// # Returns
    ///
    /// An `Option` containing the module if found, or `None` otherwise.
    fn get(&self, name: &str) -> Option<Arc<Mutex<Module>>> {
        debug!("Retrieving module from cache: {}", name);
        let cache = self.modules.lock().unwrap();
        cache.get(name).cloned()
    }
}
