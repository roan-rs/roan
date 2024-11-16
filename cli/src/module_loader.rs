use crate::context::GlobalContext;
use anstream::ColorChoice;
use roan_engine::{
    context::Context,
    module::{
        loaders::{ident::ModuleIdentifier, remove_surrounding_quotes, ModuleLoader},
        Module,
    },
    path::canonicalize_path,
    source::Source,
};
use std::collections::HashMap;
use tracing::debug;

/// A basic implementation of the `ModuleLoader` trait that caches modules in memory.
#[derive(Debug)]
pub struct RoanModuleLoader {
    modules: HashMap<String, Module>,
}

#[allow(dead_code)]
impl RoanModuleLoader {
    /// Creates a new [`RoanModuleLoader`] with an empty cache of modules.
    pub fn new() -> Self {
        debug!("Creating new BasicModuleLoader");
        Self {
            modules: HashMap::new(),
        }
    }

    /// Creates a new [`RoanModuleLoader`] with the specified cache of modules.
    pub fn with_modules(cache: HashMap<String, Module>) -> Self {
        debug!("Creating new BasicModuleLoader with provided module cache");
        Self { modules: cache }
    }

    /// Returns a clone of the module cache.
    pub fn modules(&self) -> &HashMap<String, Module> {
        &self.modules
    }
}

impl ModuleLoader for RoanModuleLoader {
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

        // If no module was found in cache we try to parse it as a module identifier.
        let resolved_path = if let Some(ident) = ModuleIdentifier::parse_module_identifier(spec) {
            let project_cwd = ctx
                .cwd
                .join("build")
                .join("deps")
                .join(ident.main_name.clone());

            let mut global = GlobalContext::from_cwd(project_cwd, ColorChoice::Auto)?;

            global.load_config()?;
            global.assert_type("lib")?;
            let parent = global.get_main_dir()?;
            canonicalize_path(parent.join(ident.file_name()))?
        } else {
            canonicalize_path(self.resolve_referrer(referrer, spec)?)?
        };

        // Use the resolved path as the cache key to prevent duplicates.
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
