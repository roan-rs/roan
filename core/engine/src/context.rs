use crate::{
    module::{
        loaders::{basic::BasicModuleLoader, ModuleLoader},
        Module,
    },
    vm::VM,
};
use anyhow::Result;
use bon::bon;
use roan_error::print_diagnostic;
use std::{cell::RefCell, fmt::Debug, path::PathBuf, rc::Rc};
use tracing::debug;

/// Struct to interact with the runtime.
///
/// # Example
/// ```rs
/// let ctx = Context::new();
/// let src_code = r#"
/// use { println, eprintln } from "std::io";
///
/// fn add(a: float, b: float) -> float {
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
    pub module_loader: Rc<RefCell<dyn ModuleLoader>>,
    pub cwd: PathBuf,
}

#[bon]
impl Context {
    /// Create a new context.
    #[builder]
    pub fn new(
        #[builder(
            default = Rc::new(RefCell::new(BasicModuleLoader::new()))
        )]
        module_loader: Rc<RefCell<dyn ModuleLoader>>,
        #[builder(default = std::env::current_dir().unwrap())] cwd: PathBuf,
    ) -> Self {
        Self { module_loader, cwd }
    }
}

impl Default for Context {
    fn default() -> Self {
        tracing::debug!("Creating default context");
        Self::builder().build()
    }
}

impl Context {
    /// Evaluate a module by parsing and interpreting it.
    ///
    /// # Arguments
    ///
    /// * `module` - The module to evaluate.
    /// * `vm` - The virtual machine instance.
    ///
    /// # Returns
    ///
    /// The result of the evaluation.
    pub fn eval(&mut self, module: &mut Module, vm: &mut VM) -> Result<()> {
        self.parse(module)?;

        self.interpret(module, vm)?;

        Ok(())
    }

    /// Parse a module to prepare it for interpretation.
    ///
    /// # Arguments
    ///
    /// * `module` - The module to parse.
    ///
    /// # Returns
    ///
    /// An empty result if successful, otherwise returns an error.
    pub fn parse(&mut self, module: &mut Module) -> Result<()> {
        match module.parse() {
            Ok(_) => Ok(()),
            Err(e) => {
                print_diagnostic(e, Some(module.source().content()));
                std::process::exit(1);
            }
        }
    }

    /// Interpret a parsed module in the virtual machine.
    ///
    /// # Arguments
    ///
    /// * `module` - The module to interpret.
    /// * `vm` - The virtual machine instance.
    ///
    /// # Returns
    ///
    /// An empty result if successful, otherwise returns an error.
    pub fn interpret(&mut self, module: &mut Module, vm: &mut VM) -> Result<()> {
        match module.interpret(self, vm) {
            Ok(_) => Ok(()),
            Err(e) => {
                print_diagnostic(e, Some(module.source().content()));
                std::process::exit(1);
            }
        }
    }

    /// Insert a module into the context.
    ///
    /// # Arguments
    /// - `name` - The name of the module.
    /// - `module` - The module to insert.
    pub fn insert_module(&mut self, name: String, module: Module) {
        debug!("Inserting module: {}", name);
        self.module_loader.borrow_mut().insert(name, module);
    }

    /// Query a module from the context.
    ///
    /// # Arguments
    /// - `name` - The name of the module to query.
    pub fn query_module(&self, name: &str) -> Option<Module> {
        self.module_loader.borrow().get(name)
    }

    /// Load a module from the context.
    ///
    /// This function is different from `query_module` in that it will attempt to load the module from the cache
    /// if it is not found it will try to resolve the path and load the module.
    ///
    /// # Arguments
    /// - `referrer` - The module that is requesting the module.
    /// - `spec` - The name of the module to load.
    pub fn load_module(&mut self, referrer: &Module, spec: &str) -> Result<Module> {
        self.module_loader.borrow_mut().load(referrer, spec, self)
    }

    pub fn module_keys(&self) -> Vec<String> {
        self.module_loader.borrow().keys()
    }

    /// Inserts or updates a module in the context.
    pub fn upsert_module(&mut self, name: String, module: Module) {
        debug!("Upserting module: {}", name);
        self.module_loader.borrow_mut().insert(name, module);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{module::Module, source::Source, value::Value, vm::VM};

    #[test]
    fn test_eval() {
        let mut ctx = Context::builder().build();
        let src_code = r#"
fn main() -> int {
    return 3;
}

main();
"#;

        let source = Source::from_string(src_code.to_string());
        let mut module = Module::new(source);

        let mut vm = VM::new();
        let result = ctx.eval(&mut module, &mut vm);

        assert_eq!(vm.pop(), Some(Value::Int(3)));
    }
}
