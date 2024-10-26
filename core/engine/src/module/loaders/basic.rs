use crate::{
    context::Context,
    module::{
        loaders::{remove_surrounding_quotes, ModuleLoader},
        Module,
    },
    path::canonicalize_path,
};
use roan_ast::source::Source;
use std::collections::HashMap;
use tracing::debug;

/// A basic implementation of the `ModuleLoader` trait that caches modules in memory.
#[derive(Debug)]
pub struct BasicModuleLoader {
    modules: HashMap<String, Module>,
}

impl BasicModuleLoader {
    /// Creates a new [`BasicModuleLoader`] with an empty cache of modules.
    pub fn new() -> Self {
        debug!("Creating new BasicModuleLoader");
        Self {
            modules: HashMap::new(),
        }
    }

    /// Creates a new [`BasicModuleLoader`] with the specified cache of modules.
    pub fn with_modules(cache: HashMap<String, Module>) -> Self {
        debug!("Creating new BasicModuleLoader with provided module cache");
        Self { modules: cache }
    }

    /// Returns a clone of the module cache.
    pub fn modules(&self) -> &HashMap<String, Module> {
        &self.modules
    }
}

impl ModuleLoader for BasicModuleLoader {
    /// Loads a module based on the specification `spec` relative to the `referrer` module.
    ///
    /// If the module is already in the cache, it returns the cached module.
    /// Otherwise, it resolves the path, loads the module, caches it, and returns it.
    fn load(&mut self, referrer: &Module, spec: &str, ctx: &Context) -> anyhow::Result<Module> {
        debug!("Loading module: {}", spec);

        // Attempt to retrieve the module from the cache.
        let cache_key = remove_surrounding_quotes(spec).to_string();
        if let Some(module) = self.modules.get(&cache_key) {
            debug!("Module found in cache: {}", cache_key);
            return Ok(module.clone());
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
        let module = Module::new(source);

        self.modules.insert(cache_key.clone(), module.clone());
        debug!("Module loaded and cached: {}", cache_key);

        Ok(module)
    }

    /// Inserts a module into the loader's cache.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the module to insert into the cache.
    /// * `module` - The module to insert into the cache.
    fn insert(&mut self, name: String, module: Module) {
        debug!("Inserting module into cache: {}", name);

        self.modules.insert(name, module);
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
    fn get(&self, name: &str) -> Option<Module> {
        debug!("Retrieving module from cache: {}", name);

        self.modules.get(remove_surrounding_quotes(name)).cloned()
    }

    /// Returns all the keys in the cache.
    ///
    /// # Returns
    /// A vector of strings representing the keys in the cache.
    fn keys(&self) -> Vec<String> {
        debug!("Retrieving keys from cache");

        self.modules.keys().cloned().collect()
    }
}
