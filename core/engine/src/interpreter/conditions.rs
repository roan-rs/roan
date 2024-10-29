use crate::{context::Context, module::Module, value::Value, vm::VM};
use roan_ast::{GetSpan, If, ThenElse};
use tracing::debug;

use anyhow::Result;
use roan_error::{error::PulseError::NonBooleanCondition, TextSpan};

impl Module {
    /// Interpret a then-else expression.
    ///
    /// # Arguments
    /// * `then_else` - [ThenElse] expression to interpret.
    /// * `ctx` - The context in which to interpret the then-else expression.
    /// * `vm` - The virtual machine to use.
    ///
    /// # Returns
    /// The result of the then-else expression.
    pub fn interpret_then_else(
        &mut self,
        then_else: ThenElse,
        ctx: &mut Context,
        vm: &mut VM,
    ) -> Result<Value> {
        debug!("Interpreting then-else");

        self.interpret_expr(&then_else.condition, ctx, vm)?;
        let condition = vm.pop().unwrap();

        let b = match condition {
            Value::Bool(b) => b,
            _ => condition.is_truthy(),
        };

        if b {
            self.interpret_expr(&then_else.then_expr, ctx, vm)?;
        } else {
            self.interpret_expr(&then_else.else_expr, ctx, vm)?;
        }

        Ok(vm.pop().expect("Expected value on stack"))
    }

    /// Interpret an if statement.
    ///
    /// # Arguments
    /// * `if_stmt` - [`If`] - The if statement to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the statement.
    pub fn interpret_if(&mut self, if_stmt: If, ctx: &mut Context, vm: &mut VM) -> Result<()> {
        debug!("Interpreting if statement");

        self.interpret_expr(&if_stmt.condition, ctx, vm)?;
        let condition_value = vm.pop().expect("Expected value on stack");

        let condition = match condition_value {
            Value::Bool(b) => b,
            Value::Null => false,
            _ => {
                return Err(NonBooleanCondition(
                    "If condition".into(),
                    TextSpan::combine(vec![if_stmt.if_token.span, if_stmt.condition.span()])
                        .unwrap(),
                )
                .into())
            }
        };

        if condition {
            self.execute_block(if_stmt.then_block, ctx, vm)?;
        } else {
            let mut executed = false;
            for else_if in if_stmt.else_ifs {
                self.interpret_expr(&else_if.condition, ctx, vm)?;
                let else_if_condition = vm.pop().expect("Expected value on stack");

                let else_if_result = match else_if_condition {
                    Value::Bool(b) => b,
                    Value::Null => false,
                    _ => {
                        return Err(NonBooleanCondition(
                            "Else if condition".into(),
                            else_if.condition.span(),
                        )
                        .into())
                    }
                };

                if else_if_result {
                    self.execute_block(else_if.block, ctx, vm)?;
                    executed = true;
                    break;
                }
            }

            if !executed {
                if let Some(else_block) = if_stmt.else_block {
                    self.execute_block(else_block.block, ctx, vm)?;
                }
            }
        }

        Ok(())
    }
}
