use crate::{
    context::Context,
    module::{
        loaders::{remove_surrounding_quotes, ModuleLoader},
        Module,
    },
    path::canonicalize_path,
};
use log::debug;
use roan_ast::source::Source;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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
    fn load(
        &self,
        referrer: &Module,
        spec: &str,
        ctx: &Context,
    ) -> anyhow::Result<Arc<Mutex<Module>>> {
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
        let resolved_path = canonicalize_path(self.resolve_referrer(referrer, spec)?)?;
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
