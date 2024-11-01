use crate::{context::Context, module::Module, value::Value, vm::VM};
use anyhow::Result;
use roan_ast::{Throw, Try};
use roan_error::error::RoanError;
use tracing::debug;

impl Module {
    /// Interpret TryCatch expression.
    ///
    /// # Arguments
    /// * `try_catch` - TryCatch expression to interpret.
    /// * `ctx` - The context in which to interpret the TryCatch expression.
    ///
    /// # Returns
    /// The result of the TryCatch expression.
    pub fn interpret_try(&mut self, try_stmt: Try, ctx: &mut Context, vm: &mut VM) -> Result<()> {
        debug!("Interpreting try");

        let try_result = self.execute_block(try_stmt.try_block.clone(), ctx, vm);

        match try_result {
            Ok(_) => return Ok(()),
            Err(e) => match e.downcast_ref::<RoanError>() {
                Some(RoanError::Throw(msg, _)) => {
                    self.enter_scope();

                    let var_name = try_stmt.error_ident.literal();
                    self.declare_variable(var_name, Value::String(msg.clone()));
                    let result = self.execute_block(try_stmt.catch_block, ctx, vm);
                    self.exit_scope();

                    result?
                }
                _ => return Err(e),
            },
        }

        Ok(())
    }

    /// Interpret a throw statement.
    ///
    /// # Arguments
    /// * `throw_stmt` - The throw statement to interpret.
    /// * `ctx` - The context in which to interpret the statement.
    ///
    /// # Returns
    /// The result of the throw statement.
    pub fn interpret_throw(&mut self, throw: Throw, ctx: &mut Context, vm: &mut VM) -> Result<()> {
        debug!("Interpreting throw");

        self.interpret_expr(&throw.value, ctx, vm)?;
        let val = vm.pop().unwrap();

        return Err(RoanError::Throw(val.to_string(), Vec::from(vm.frames())).into());
    }
}
