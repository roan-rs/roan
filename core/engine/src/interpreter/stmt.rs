use crate::{
    context::Context,
    module::{ExportType, Module, StoredFunction},
    value::Value,
    vm::VM,
};
use anyhow::Result;
use roan_ast::{
    Block, Fn, GetSpan, If, Let, Loop, Stmt, Struct, StructImpl, Token, TraitDef, TraitImpl, Use,
    While,
};
use roan_error::{
    error::{
        PulseError,
        PulseError::{ImportError, ModuleNotFoundError, NonBooleanCondition},
    },
    print_diagnostic, TextSpan,
};
use std::sync::{Arc, Mutex};
use tracing::debug;

impl Module {
    /// Interpret statement from the module.
    ///
    /// # Arguments
    /// * `stmt` - [`Stmt`] - The statement to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the statement.
    pub fn interpret_stmt(&mut self, stmt: Stmt, ctx: &Context, vm: &mut VM) -> Result<()> {
        match stmt {
            Stmt::Fn(f) => self.interpret_function(f)?,
            Stmt::Use(u) => self.interpret_use(u, ctx, vm)?,
            Stmt::While(while_stmt) => self.interpret_while(while_stmt, ctx, vm)?,
            Stmt::Loop(loop_stmt) => self.interpret_loop(loop_stmt, ctx, vm)?,
            Stmt::Block(block) => self.execute_block(block, ctx, vm)?,
            Stmt::If(if_stmt) => self.interpret_if(if_stmt, ctx, vm)?,
            Stmt::Break(token) => {
                debug!("Interpreting break statement");
                return Err(PulseError::LoopBreak(token.span).into());
            }
            Stmt::Continue(token) => {
                debug!("Interpreting continue statement");
                return Err(PulseError::LoopContinue(token.span).into());
            }
            Stmt::Throw(throw) => {
                debug!("Interpreting throw");

                self.interpret_expr(&throw.value, ctx, vm)?;
                let val = vm.pop().unwrap();

                return Err(PulseError::Throw(val.to_string(), Vec::from(vm.frames())).into());
            }
            Stmt::Try(try_stmt) => {
                debug!("Interpreting try");

                let try_result = self.execute_block(try_stmt.try_block.clone(), ctx, vm);

                match try_result {
                    Ok(_) => return Ok(()),
                    Err(e) => match e.downcast_ref::<PulseError>() {
                        Some(PulseError::Throw(msg, _)) => {
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
            }
            Stmt::Let(l) => self.interpret_let(l, vm, ctx)?,
            Stmt::Expr(expr) => {
                debug!("Interpreting expression");

                self.interpret_expr(expr.as_ref(), ctx, vm)?;
            }
            Stmt::Return(r) => {
                debug!("Interpreting return: {:?}", r);

                if let Some(expr) = r.expr {
                    self.interpret_expr(expr.as_ref(), ctx, vm)?;
                } else {
                    vm.push(Value::Void);
                }
            }
            Stmt::Struct(struct_stmt) => {
                self.structs.push(struct_stmt.clone());

                if struct_stmt.public {
                    self.exports.push((
                        struct_stmt.name.literal(),
                        ExportType::Struct(struct_stmt.clone()),
                    ));
                }
            }
            Stmt::TraitDef(trait_stmt) => {
                self.traits.push(trait_stmt.clone());

                if trait_stmt.public {
                    self.exports.push((
                        trait_stmt.name.literal(),
                        ExportType::Trait(trait_stmt.clone()),
                    ));
                }
            }
            Stmt::StructImpl(impl_stmt) => self.interpret_struct_impl(impl_stmt)?,
            Stmt::TraitImpl(impl_stmt) => self.interpret_trait_impl(impl_stmt)?,
        }

        Ok(())
    }

    /// Interpret a struct implementation.
    ///
    /// # Arguments
    /// * `impl_stmt` - [`StructImpl`] - The struct implementation to interpret.
    pub fn interpret_struct_impl(&mut self, impl_stmt: StructImpl) -> Result<()> {
        let struct_name = impl_stmt.struct_name.literal();

        let mut struct_def = self.get_struct(&struct_name, impl_stmt.struct_name.span.clone())?;
        struct_def.impls.push(impl_stmt.clone());

        if let Some(existing_struct) = self
            .structs
            .iter_mut()
            .find(|s| s.name.literal() == struct_name)
        {
            *existing_struct = struct_def;

            if let Some(export) = self.exports.iter_mut().find(|(n, _)| n == &struct_name) {
                if let ExportType::Struct(s) = &mut export.1 {
                    s.impls.push(impl_stmt);
                }
            }
        }

        Ok(())
    }

    /// Interpret a trait implementation.
    ///
    /// # Arguments
    /// * `impl_stmt` - [`TraitImpl`] - The trait implementation to interpret.
    pub fn interpret_trait_impl(&mut self, impl_stmt: TraitImpl) -> Result<()> {
        let for_name = impl_stmt.struct_name.literal();
        let trait_name = impl_stmt.trait_name.literal();

        let mut struct_def = self.get_struct(&for_name, impl_stmt.struct_name.span.clone())?;
        let trait_def = self.get_trait(&trait_name, impl_stmt.trait_name.span.clone())?;

        if struct_def
            .trait_impls
            .iter()
            .any(|t| t.trait_name.literal() == trait_name)
        {
            return Err(PulseError::StructAlreadyImplementsTrait(
                for_name,
                trait_name,
                impl_stmt.trait_name.span.clone(),
            )
            .into());
        }

        let missing_methods: Vec<String> = trait_def
            .methods
            .iter()
            .filter(|m| !impl_stmt.methods.iter().any(|i| i.name == m.name))
            .map(|m| m.name.clone())
            .collect();

        if !missing_methods.is_empty() {
            return Err(PulseError::TraitMethodNotImplemented(
                trait_name,
                missing_methods,
                impl_stmt.trait_name.span.clone(),
            )
            .into());
        }

        struct_def.trait_impls.push(impl_stmt.clone());

        if let Some(existing_struct) = self
            .structs
            .iter_mut()
            .find(|s| s.name.literal() == for_name)
        {
            *existing_struct = struct_def;

            if let Some(export) = self.exports.iter_mut().find(|(n, _)| n == &for_name) {
                if let ExportType::Struct(s) = &mut export.1 {
                    s.trait_impls.push(impl_stmt);
                }
            }
        }

        Ok(())
    }

    pub fn get_trait(&self, name: &str, span: TextSpan) -> Result<TraitDef> {
        Ok(self
            .traits
            .iter()
            .find(|t| t.name.literal() == name)
            .cloned()
            .ok_or_else(|| PulseError::TraitNotFoundError(name.into(), span))?)
    }

    pub fn get_struct(&self, name: &str, span: TextSpan) -> Result<Struct> {
        Ok(self
            .structs
            .iter()
            .find(|s| s.name.literal() == name)
            .cloned()
            .ok_or_else(|| PulseError::StructNotFoundError(name.into(), span))?)
    }

    /// Interpret a let statement.
    ///
    /// # Arguments
    /// * `let_stmt` - [`Let`] - The let statement to interpret.
    /// * `vm` - [`VM`] - The virtual machine to use for interpretation.
    /// * `ctx` - [`Context`] - The context in which to interpret the statement.
    pub fn interpret_let(&mut self, l: Let, vm: &mut VM, ctx: &Context) -> Result<()> {
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
                        return Err(PulseError::TypeMismatch(
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
    /// [PulseError::LoopBreak] and [PulseError::LoopContinue] are handled if they are inside a loop otherwise they are returned as an error.
    ///
    /// # Arguments
    /// * `result` - [Result<()>] - The result to handle.
    pub fn handle_loop_result(&mut self, result: Result<()>) -> Result<()> {
        match result {
            Ok(_) => {}
            Err(e) => match e.downcast::<PulseError>() {
                Ok(PulseError::LoopBreak(_)) => {}
                Ok(PulseError::LoopContinue(_)) => {}
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
    pub fn interpret_loop(&mut self, loop_stmt: Loop, ctx: &Context, vm: &mut VM) -> Result<()> {
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
    pub fn interpret_while(&mut self, while_stmt: While, ctx: &Context, vm: &mut VM) -> Result<()> {
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
    pub fn interpret_function(&mut self, function: Fn) -> Result<()> {
        debug!("Interpreting function: {}", function.name);
        self.functions.push(StoredFunction::Function {
            function: function.clone(),
            defining_module: Arc::clone(&Arc::new(Mutex::new(self.clone()))),
        });

        if function.public {
            self.exports.push((
                function.name.clone(),
                ExportType::Function(function.clone()),
            ));
        }

        Ok(())
    }

    /// Interpret an use statement.
    ///
    /// # Arguments
    /// * `use_stmt` - [`Use`] - The use statement to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the statement.
    pub fn interpret_use(&mut self, u: Use, ctx: &Context, vm: &mut VM) -> Result<()> {
        debug!("Interpreting use: {}", u.from.literal());

        // Load the module as an Arc<Mutex<Module>>
        let module = ctx
            .module_loader
            .load(&self.clone(), &u.from.literal(), ctx)
            .map_err(|_| ModuleNotFoundError(u.from.literal().to_string(), u.from.span.clone()))?;

        // Lock the loaded module for parsing and interpretation
        let mut loaded_module = module.lock().expect("Failed to lock loaded module");

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
                            defining_module: Arc::clone(&module),
                        });
                    }
                    ExportType::Struct(s) => {
                        self.structs.push(s.clone());
                    }
                    ExportType::Trait(t) => {
                        self.traits.push(t.clone());
                    }
                }
            } else {
                return Err(ImportError(name, item.span.clone()).into());
            }
        }

        Ok(())
    }

    /// Interpret an if statement.
    ///
    /// # Arguments
    /// * `if_stmt` - [`If`] - The if statement to interpret.
    /// * `ctx` - [`Context`] - The context in which to interpret the statement.
    pub fn interpret_if(&mut self, if_stmt: If, ctx: &Context, vm: &mut VM) -> Result<()> {
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

    /// Execute a block of statements within a new scope.
    ///
    /// # Arguments
    /// * `block` - [`Block`] - The block of statements to execute.
    pub fn execute_block(&mut self, block: Block, ctx: &Context, vm: &mut VM) -> Result<()> {
        debug!("Interpreting block statement");

        self.enter_scope();
        for stmt in block.stmts {
            self.interpret_stmt(stmt, ctx, vm)?;
        }
        self.exit_scope();
        Ok(())
    }
}
