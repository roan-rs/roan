use crate::module::{loader::{ModuleLoader, BasicModuleLoader}, Module};
use anyhow::Result;
use bon::bon;
use std::{fmt::Debug, rc::Rc};
use log::debug;

/// Struct to interact with the runtime.
///
/// # Example
/// ```rs
/// let ctx = Context::new();
/// let src_code = r#"
/// use { println, eprintln } from "std::io";
///
/// export fn add(a: float, b: float) -> float {
///     return a + b;
/// }
///
/// fn main() -> int {
///     let i = 3.14;
///     let j = true;
///
///     if j {
///         i = add(i, 2.0);
///     } else {
///         eprintln("Goodbye, world!");
///     }
///
///     return 0;
/// }
///
/// main();
/// "#;
///
/// let source = Source::from_string(src_code);
/// let module = Module::from_source(source, ctx)?;
///
/// let result = ctx.eval(module);
///
/// assert_eq!(result, Ok(Value::Int(3)));
/// ```
#[derive(Clone, Debug)]
pub struct Context {
    pub module_loader: Rc<dyn ModuleLoader>,
}

#[bon]
impl Context {
    /// Create a new context.
    #[builder]
    pub fn new(
        #[builder(
            default = Rc::new(BasicModuleLoader::new())
        )]
        module_loader: Rc<dyn ModuleLoader>,
    ) -> Self {
        Self { module_loader }
    }
}

impl Default for Context {
    fn default() -> Self {
        log::debug!("Creating default context");
        Self::builder().build()
    }
}

impl Context {
    /// Evaluate a module.
    ///
    /// # Arguments
    ///
    /// * `module` - The module to evaluate.
    ///
    /// # Returns
    ///
    /// The result of the evaluation.
    pub fn eval(&self, mut module: Module) -> Result<()> {
        debug!("Evaluating module: {:?}", module);
        module.parse()?;
        module.interpret(self)?;

        Ok(())
    }

    /// Insert a module into the context.
    ///
    /// # Arguments
    /// - `name` - The name of the module.
    /// - `module` - The module to insert.
    pub fn insert_module(&self, name: String, module: Module) {
        debug!("Inserting module: {}", name);
        self.module_loader.insert(name, module);
    }

    /// Get a module from the context.
    ///
    /// # Arguments
    /// - `name` - The name of the module.
    pub fn get_module(&self, name: String) -> Option<Module> {
        debug!("Getting module: {}", name);
        self.module_loader.get(name)
    }
}

