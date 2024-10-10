use std::sync::{Arc, Mutex};
use log::debug;
use roan_ast::{Block, GetSpan, If, Stmt, Token, Fn, Use, While, Loop};
use roan_error::error::PulseError;
use roan_error::error::PulseError::{ImportError, ModuleNotFoundError, NonBooleanCondition};
use roan_error::{print_diagnostic, TextSpan};
use crate::context::Context;
use crate::module::{ExportType, Module, StoredFunction};
use crate::value::Value;
use anyhow::Result;

impl Module {
    /// Interpret statement from the module.
    ///
    /// # Arguments
    /// * `stmt` - [`Stmt`] - The statement to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the statement.
    pub fn interpret_stmt(&mut self, stmt: Stmt, ctx: &Context) -> Result<()> {
        match stmt {
            Stmt::Fn(f) => self.interpret_function(f)?,
            Stmt::Use(u) => self.interpret_use(u, ctx)?,
            Stmt::While(while_stmt) => self.interpret_while(while_stmt, ctx)?,
            Stmt::Loop(loop_stmt) => self.interpret_loop(loop_stmt, ctx)?,
            Stmt::Block(block) => self.execute_block(block, ctx)?,
            Stmt::If(if_stmt) => self.interpret_if(if_stmt, ctx)?,
            Stmt::Break(token) => {
                debug!("Interpreting break statement");
                return Err(PulseError::LoopBreak(token.span).into());
            }
            Stmt::Continue(token) => {
                debug!("Interpreting continue statement");
                return Err(PulseError::LoopContinue(token.span).into());
            }
            Stmt::Throw(throw) => {
                debug!("Interpreting throw: {:?}", throw);

                self.interpret_expr(&throw.value, ctx)?;
                let val = self.vm.pop().unwrap();

                return Err(PulseError::Throw(val.to_string(), Vec::from(self.vm.frames())).into());
            }
            Stmt::Try(try_stmt) => {
                debug!("Interpreting try: {:?}", try_stmt);
            }
            Stmt::Let(l) => {
                debug!("Interpreting let: {:?}", l.ident);
                self.interpret_expr(l.initializer.as_ref(), ctx)?;

                let val = self.vm.pop().unwrap();
                let ident = l.ident.literal();
                self.declare_variable(ident.clone(), val);
            }
            Stmt::Expr(expr) => {
                debug!("Interpreting expression: {:?}", expr);

                self.interpret_expr(expr.as_ref(), ctx)?;
            }
            Stmt::Return(r) => {
                debug!("Interpreting return: {:?}", r);

                if let Some(expr) = r.expr {
                    self.interpret_expr(expr.as_ref(), ctx)?;
                }
            }
        }

        Ok(())
    }

    /// Interpret a loop statement.
    ///
    /// # Arguments
    /// * `loop_stmt` - [`Loop`] - The loop to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the loop.
    pub fn interpret_loop(&mut self, loop_stmt: Loop, ctx: &Context) -> Result<()> {
        debug!("Interpreting infinite loop");
        loop {
            self.enter_scope();
            let result = self.execute_block(loop_stmt.block.clone(), ctx);
            self.exit_scope();

            match result {
                Ok(_) => {}
                Err(e) => match e.downcast::<PulseError>() {
                    Ok(PulseError::LoopBreak(_)) => break,
                    Ok(PulseError::LoopContinue(_)) => continue,
                    Ok(other) => return Err(other.into()),
                    Err(e) => return Err(e),
                },
            }
        }

        Ok(())
    }

    /// Interpret a while loop.
    ///
    /// # Arguments
    /// * `while_stmt` - [`While`] - The while loop to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the while loop.
    pub fn interpret_while(&mut self, while_stmt: While, ctx: &Context) -> Result<()> {
        debug!("Interpreting while loop");

        loop {
            self.interpret_expr(&while_stmt.condition, ctx)?;
            let condition_value = self.vm.pop().expect("Expected value on stack");

            let condition = match condition_value {
                Value::Bool(b) => b,
                _ => {
                    return Err(NonBooleanCondition(
                        "While loop condition".into(),
                        while_stmt.condition.span(),
                    )
                        .into())
                }
            };

            if !condition {
                break;
            }

            self.enter_scope();
            let result = self.execute_block(while_stmt.block.clone(), ctx);
            self.exit_scope();

            match result {
                Ok(_) => {}
                Err(e) => match e.downcast::<PulseError>() {
                    Ok(PulseError::LoopBreak(_)) => break,
                    Ok(PulseError::LoopContinue(_)) => continue,
                    Ok(other) => return Err(other.into()),
                    Err(e) => return Err(e),
                },
            }
        }

        Ok(())
    }

    /// Interpret a function declaration.
    ///
    /// # Arguments
    /// * `function` - [`Fn`] - The function to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the function.
    pub fn interpret_function(&mut self, function: Fn) -> Result<()> {
        debug!("Interpreting function: {}", function.name);
        self.functions.push(StoredFunction::Function {
            function: function.clone(),
            defining_module: Arc::clone(&Arc::new(Mutex::new(self.clone()))),
        });

        if function.exported {
            self.exports
                .push((function.name.clone(), ExportType::Function(function.clone())));
        }

        Ok(())
    }

    /// Interpret an use statement.
    ///
    /// # Arguments
    /// * `use_stmt` - [`Use`] - The use statement to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the statement.
    pub fn interpret_use(&mut self, u: Use, ctx: &Context) -> Result<()> {
        debug!("Interpreting use: {}", u.from.literal());

        // Load the module as an Arc<Mutex<Module>>
        let module = ctx
            .module_loader
            .load(&self.clone(), &u.from.literal(), ctx)
            .map_err(|_| {
                ModuleNotFoundError(u.from.literal().to_string(), u.from.span.clone())
            })?;

        // Lock the loaded module for parsing and interpretation
        let mut loaded_module = module.lock().expect("Failed to lock loaded module");

        match loaded_module.parse() {
            Ok(_) => {}
            Err(e) => {
                print_diagnostic(e, Some(loaded_module.source().content()));
                std::process::exit(1);
            }
        }

        match loaded_module.interpret(ctx) {
            Ok(_) => {}
            Err(e) => {
                print_diagnostic(e, Some(loaded_module.source().content()));
                std::process::exit(1);
            }
        }

        // Collect the items to import
        let imported_items: Vec<(String, &Token)> =
            u.items.iter().map(|i| (i.literal(), i)).collect();

        for (name, item) in imported_items {
            match loaded_module.find_function(&name) {
                Some(StoredFunction::Function {
                         function,
                         defining_module,
                     }) => {
                    self.functions.push(StoredFunction::Function {
                        function: function.clone(),
                        defining_module: Arc::clone(&defining_module),
                    });
                }
                Some(StoredFunction::Native(n)) => {
                    self.functions.push(StoredFunction::Native(n.clone()));
                }
                None => {
                    return Err(ImportError(name, item.span.clone()).into());
                }
            }
        }

        Ok(())
    }

    /// Interpret an if statement.
    ///
    /// # Arguments
    /// * `if_stmt` - [`If`] - The if statement to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the statement.
    pub fn interpret_if(&mut self, if_stmt: If, ctx: &Context) -> Result<()> {
        debug!("Interpreting if statement");

        self.interpret_expr(&if_stmt.condition, ctx)?;
        let condition_value = self.vm.pop().expect("Expected value on stack");

        let condition = match condition_value {
            Value::Bool(b) => b,
            _ => {
                return Err(NonBooleanCondition(
                    "If condition".into(),
                    TextSpan::combine(vec![if_stmt.if_token.span, if_stmt.condition.span()]),
                )
                    .into())
            }
        };

        if condition {
            self.execute_block(if_stmt.then_block, ctx)?;
        } else {
            let mut executed = false;
            for else_if in if_stmt.else_ifs {
                self.interpret_expr(&else_if.condition, ctx)?;
                let else_if_condition = self.vm.pop().expect("Expected value on stack");

                let else_if_result = match else_if_condition {
                    Value::Bool(b) => b,
                    _ => {
                        return Err(NonBooleanCondition(
                            "Else if condition".into(),
                            else_if.condition.span(),
                        )
                            .into())
                    }
                };

                if else_if_result {
                    self.execute_block(else_if.block, ctx)?;
                    executed = true;
                    break;
                }
            }

            if !executed {
                if let Some(else_block) = if_stmt.else_block {
                    self.execute_block(else_block.block, ctx)?;
                }
            }
        }

        Ok(())
    }

    /// Execute a block of statements within a new scope.
    ///
    /// # Arguments
    /// * `block` - [`Block`] - The block of statements to execute.
    pub fn execute_block(&mut self, block: Block, ctx: &Context) -> Result<()> {
        debug!("Interpreting block statement");

        self.enter_scope();
        for stmt in block.stmts {
            self.interpret_stmt(stmt, ctx)?;
        }
        self.exit_scope();
        Ok(())
    }
}