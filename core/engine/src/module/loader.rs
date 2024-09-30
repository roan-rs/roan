use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Component, PathBuf};
use std::rc::Rc;
use crate::module::Module;
use anyhow::Result;
use roan_ast::source::Source;
use roan_error::error::PulseError::ModuleError;
use crate::context::Context;

/// Trait that defines the interface for a module loader.
pub trait ModuleLoader {
    /// Load a module from a given source.
    fn load(&self, referrer: &Module, spec: &str, ctx: Context) -> Result<Module>;


    /// Resolves the path of a referenced module based on the referrer module's path and the provided specification.
    ///
    /// This function constructs a `PathBuf` for the given `spec` relative to the `referrer` module's path.
    /// If `spec` is an absolute path, it returns it directly. Otherwise, it joins the `spec` with the parent
    /// directory of the `referrer` module's path. On Windows, it replaces forward slashes in `spec` with
    /// backslashes to ensure the path is formatted correctly.
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
        let referrer_path = referrer.path().map_or_else(|| PathBuf::new(), |p| p.to_path_buf());
        let dir = referrer_path.parent().expect("Module path has no parent");

        let w = spec.replace("/", "\\");
        let str_path = remove_surrounding_quotes(if cfg!(windows) {
            &w
        } else {
            spec
        });

        let mut spec_path = PathBuf::from(str_path);

        let path = if spec_path.is_absolute() {
            spec_path
        } else {
            dir.join(spec_path)
        };

        Ok(path)
    }
}

fn remove_surrounding_quotes(s: &str) -> &str {
    if s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

#[derive(Clone, Debug)]
pub struct BasicModuleLoader {
    modules: Rc<RefCell<HashMap<String, Module>>>,
}

impl BasicModuleLoader {
    /// Creates a new [`BasicModuleLoader`] with an empty cache of modules.
    pub fn new() -> Self {
        Self {
            modules: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    /// Creates a new [`BasicModuleLoader`] with the specified cache of modules.
    pub fn with_modules(cache: Rc<RefCell<HashMap<String, Module>>>) -> Self {
        Self { modules: cache }
    }

    /// Returns the cache of modules.
    pub fn modules(&self) -> Rc<RefCell<HashMap<String, Module>>> {
        self.modules.clone()
    }
}

impl ModuleLoader for BasicModuleLoader {
    fn load(&self, referrer: &Module, spec: &str, ctx: Context) -> Result<Module> {
        let modules = self.modules.borrow();
        if let Some(module) = modules.get(spec).cloned() {
            return Ok(module);
        }
        drop(modules);

        let path = self.resolve_referrer(referrer, spec)?;
        let source = Source::from_path(path)?;
        let module = Module::new(source, ctx.clone());

        self.modules.borrow_mut().insert(spec.to_string(), module.clone());

        Ok(module)
    }
}