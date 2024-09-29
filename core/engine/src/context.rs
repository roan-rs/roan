use std::fmt::Debug;
use std::rc::Rc;
use crate::module::{loader::ModuleLoader, fs::FsModuleLoader};
use bon::bon;
use anyhow::Result;

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
            default = Rc::new(FsModuleLoader::new(".").unwrap())
        )] module_loader: Rc<dyn ModuleLoader>
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
    pub fn eval(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::loader::ModuleLoader;
    use std::rc::Rc;

    struct MockLoader;

    impl ModuleLoader for MockLoader {
        fn load(&self) {
            println!("Loading module");
        }
    }

    #[test]
    fn test_context() {
        let loader = Rc::new(MockLoader);
        let ctx = Context::builder().module_loader(loader).build();

        assert_eq!(format!("{:?}", ctx), "Context { module_loader: \"ModuleLoader\" }");
    }
}