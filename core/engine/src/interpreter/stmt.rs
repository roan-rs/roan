use crate::{
    context::Context,
    module::{loaders::remove_surrounding_quotes, ExportType, Module, StoredConst, StoredFunction},
    value::Value,
    vm::VM,
};
use anyhow::Result;
use roan_ast::{Block, Fn, GetSpan, Let, Loop, Stmt, Token, Use, While};
use roan_error::{
    error::{
        RoanError,
        RoanError::{FailedToImportModule, ImportError, NonBooleanCondition},
    },
    print_diagnostic, TextSpan,
};
use tracing::debug;

impl Module {
    /// Interpret statement from the module.
    ///
    /// # Arguments
    /// * `stmt` - [`Stmt`] - The statement to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the statement.
    pub fn interpret_stmt(&mut self, stmt: Stmt, ctx: &mut Context, vm: &mut VM) -> Result<()> {
        match stmt {
            Stmt::Fn(f) => self.interpret_function(f, ctx)?,
            Stmt::Use(u) => self.interpret_use(u, ctx, vm)?,
            Stmt::While(while_stmt) => self.interpret_while(while_stmt, ctx, vm)?,
            Stmt::Loop(loop_stmt) => self.interpret_loop(loop_stmt, ctx, vm)?,
            Stmt::Block(block) => self.execute_block(block, ctx, vm)?,
            Stmt::If(if_stmt) => self.interpret_if(if_stmt, ctx, vm)?,
            Stmt::Break(token) => {
                debug!("Interpreting break statement");
                return Err(RoanError::LoopBreak(token.span).into());
            }
            Stmt::Continue(token) => {
                debug!("Interpreting continue statement");
                return Err(RoanError::LoopContinue(token.span).into());
            }
            Stmt::Throw(throw) => self.interpret_throw(throw, ctx, vm)?,
            Stmt::Try(try_stmt) => self.interpret_try(try_stmt, ctx, vm)?,
            Stmt::Let(l) => self.interpret_let(l, vm, ctx)?,
            Stmt::Expr(expr) => self.interpret_expr(expr.as_ref(), ctx, vm)?,
            Stmt::Return(r) => {
                debug!("Interpreting return: {:?}", r);

                if let Some(expr) = r.expr {
                    self.interpret_expr(expr.as_ref(), ctx, vm)?;
                } else {
                    vm.push(Value::Void);
                }
            }
            Stmt::Struct(struct_stmt) => self.interpret_struct(struct_stmt, ctx)?,
            Stmt::TraitDef(trait_stmt) => self.interpret_trait(trait_stmt, ctx)?,
            Stmt::StructImpl(impl_stmt) => self.interpret_struct_impl(impl_stmt, ctx)?,
            Stmt::TraitImpl(impl_stmt) => self.interpret_trait_impl(impl_stmt, ctx)?,
            Stmt::Const(c) => {
                let def_expr = c.expr.clone();
                let ident_literal = c.ident.literal();
                let is_public = c.public;

                self.interpret_expr(&def_expr, ctx, vm)?;

                let val = vm.pop().expect("Expected value on stack");

                let stored_val = StoredConst {
                    ident: c.ident.clone(),
                    value: val.clone(),
                };

                self.consts.push(stored_val.clone());

                if is_public {
                    self.exports
                        .push((ident_literal, ExportType::Const(stored_val)));
                }
            }
        }

        Ok(())
    }

    /// Interpret a let statement.
    ///
    /// # Arguments
    /// * `let_stmt` - [`Let`] - The let statement to interpret.
    /// * `vm` - [`VM`] - The virtual machine to use for interpretation.
    /// * `ctx` - [`Context`] - The context in which to interpret the statement.
    pub fn interpret_let(&mut self, l: Let, vm: &mut VM, ctx: &mut Context) -> Result<()> {
        debug!("Interpreting let: {:?}", l.ident);
        self.interpret_expr(l.initializer.as_ref(), ctx, vm)?;

        let val = vm.pop().unwrap();
        let ident = l.ident.literal();

        if let Some(type_annotation) = &l.type_annotation {
            let type_name = type_annotation.type_name.literal();

            if type_annotation.is_array {
                match val.clone() {
                    Value::Vec(v) => {
                        // TODO: actually display what part of the array is wrong
                        for item in v.iter() {
                            item.check_type(&type_name, l.initializer.span())?
                        }
                    }
                    _ => {
                        return Err(RoanError::TypeMismatch(
                            format!(
                                "Expected array of type {} but got {}",
                                type_name,
                                val.type_name()
                            ),
                            l.initializer.span(),
                        )
                        .into());
                    }
                }
            } else {
                if val.is_null() && type_annotation.is_nullable {
                    self.declare_variable(ident.clone(), val);
                    return Ok(());
                }

                val.check_type(
                    &type_name,
                    TextSpan::combine(vec![
                        l.ident.span,
                        type_annotation.type_name.span.clone(),
                        l.initializer.span(),
                    ])
                    .unwrap(),
                )?
            }
        }

        self.declare_variable(ident.clone(), val);

        Ok(())
    }

    /// Handle the result of a loop statement.
    ///
    /// [RoanError::LoopBreak] and [RoanError::LoopContinue] are handled if they are inside a loop otherwise they are returned as an error.
    ///
    /// # Arguments
    /// * `result` - [Result<()>] - The result to handle.
    pub fn handle_loop_result(&mut self, result: Result<()>) -> Result<()> {
        match result {
            Ok(_) => {}
            Err(e) => match e.downcast::<RoanError>() {
                Ok(RoanError::LoopBreak(_)) => {}
                Ok(RoanError::LoopContinue(_)) => {}
                Ok(other) => return Err(other.into()),
                Err(e) => return Err(e),
            },
        }

        Ok(())
    }

    /// Interpret a loop statement.
    ///
    /// # Arguments
    /// * `loop_stmt` - [`Loop`] - The loop to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the loop.
    pub fn interpret_loop(
        &mut self,
        loop_stmt: Loop,
        ctx: &mut Context,
        vm: &mut VM,
    ) -> Result<()> {
        debug!("Interpreting infinite loop");
        loop {
            self.enter_scope();
            let result = self.execute_block(loop_stmt.block.clone(), ctx, vm);
            self.exit_scope();

            self.handle_loop_result(result)?
        }
    }

    /// Interpret a while loop.
    ///
    /// # Arguments
    /// * `while_stmt` - [`While`] - The while loop to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the while loop.
    pub fn interpret_while(
        &mut self,
        while_stmt: While,
        ctx: &mut Context,
        vm: &mut VM,
    ) -> Result<()> {
        debug!("Interpreting while loop");

        loop {
            self.interpret_expr(&while_stmt.condition, ctx, vm)?;
            let condition_value = vm.pop().expect("Expected value on stack");

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
            let result = self.execute_block(while_stmt.block.clone(), ctx, vm);
            self.exit_scope();

            self.handle_loop_result(result)?
        }

        Ok(())
    }

    /// Interpret a function declaration.
    ///
    /// # Arguments
    /// * `function` - [`Fn`] - The function to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the function.
    pub fn interpret_function(&mut self, function: Fn, ctx: &mut Context) -> Result<()> {
        debug!("Interpreting function: {}", function.name);

        self.functions.push(StoredFunction::Function {
            function: function.clone(),
            defining_module: self.id(),
        });

        if function.public {
            self.exports.push((
                function.name.clone(),
                ExportType::Function(function.clone()),
            ));
        }

        ctx.upsert_module(self.id().clone(), self.clone());
        Ok(())
    }

    /// Interpret an use statement.
    ///
    /// # Arguments
    /// * `use_stmt` - [`Use`] - The use statement to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the statement.
    pub fn interpret_use(&mut self, u: Use, ctx: &mut Context, vm: &mut VM) -> Result<()> {
        debug!("Interpreting use: {}", u.from.literal());

        let mut loaded_module = ctx
            .load_module(&self.clone(), remove_surrounding_quotes(&u.from.literal()))
            .map_err(|err| {
                FailedToImportModule(
                    u.from.literal().to_string(),
                    err.to_string(),
                    u.from.span.clone(),
                )
            })?;

        match loaded_module.parse() {
            Ok(_) => {}
            Err(e) => {
                print_diagnostic(e, Some(loaded_module.source().content()));
                std::process::exit(1);
            }
        }

        match loaded_module.interpret(ctx, vm) {
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
            let export = loaded_module.exports.iter().find(|(n, _)| n == &name);

            if let Some((name, value)) = export {
                debug!("Importing {} from {}", name, u.from.literal());
                match value {
                    ExportType::Function(f) => {
                        self.functions.push(StoredFunction::Function {
                            function: f.clone(),
                            defining_module: loaded_module.id(),
                        });
                    }
                    ExportType::Struct(s) => {
                        self.structs.push(s.clone());
                    }
                    ExportType::Trait(t) => {
                        self.traits.push(t.clone());
                    }
                    ExportType::Const(c) => {
                        self.consts.push(c.clone());
                    }
                }
            } else {
                return Err(ImportError(name, item.span.clone()).into());
            }
        }

        Ok(())
    }

    /// Execute a block of statements within a new scope.
    ///
    /// # Arguments
    /// * `block` - [`Block`] - The block of statements to execute.
    pub fn execute_block(&mut self, block: Block, ctx: &mut Context, vm: &mut VM) -> Result<()> {
        debug!("Interpreting block statement");

        self.enter_scope();
        for stmt in block.stmts {
            self.interpret_stmt(stmt, ctx, vm)?;
        }
        self.exit_scope();
        Ok(())
    }
}
