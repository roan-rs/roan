use crate::module::{loader::{ModuleLoader, BasicModuleLoader}, Module};
use anyhow::Result;
use bon::bon;
use std::{fmt::Debug, rc::Rc};


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
#[derive(Clone)]
pub struct Context {
    pub module_loader: Rc<dyn ModuleLoader>,
}

impl Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("module_loader", &"ModuleLoader")
            .finish()
    }
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
        module.parse()?;

        module.interpret()?;

        Ok(())
    }
}

