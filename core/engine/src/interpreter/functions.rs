use crate::{
    context::Context,
    module::{Module, StoredFunction},
    value::Value,
    vm::{native_fn::NativeFunction, VM},
};
use anyhow::Result;
use roan_ast::{CallExpr, Expr, GetSpan};
use roan_error::{
    error::PulseError::{InvalidSpread, MissingParameter, TypeMismatch, UndefinedFunctionError},
    frame::Frame,
    print_diagnostic, TextSpan,
};
use tracing::debug;

impl Module {
    /// Executes a native function with the provided arguments.
    pub fn execute_native_function(
        &mut self,
        mut native: NativeFunction,
        args: Vec<Value>,
        vm: &mut VM,
    ) -> Result<()> {
        let result = native.call(args)?;
        vm.push(result);

        Ok(())
    }

    /// Executes a user-defined function with the provided arguments.
    pub fn execute_user_defined_function(
        &mut self,
        function: roan_ast::Fn,
        def_module: &mut Module,
        args: Vec<Value>,
        ctx: &mut Context,
        vm: &mut VM,
        call: &CallExpr,
    ) -> Result<()> {
        debug!("Executing user-defined function: {}", function.name);

        self.enter_scope();

        {
            let exprs = call
                .args
                .iter()
                .map(|arg| arg.span())
                .collect::<Vec<TextSpan>>();

            let mut offset = 0;
            for (i, (param, arg)) in function
                .params
                .iter()
                .zip(args.iter().chain(std::iter::repeat(&Value::Null)))
                .enumerate()
            {
                let ident = param.ident.literal();
                // Maybe we could find a better way to handle this
                if ident == "self" {
                    offset += 1;

                    def_module.declare_variable(ident, arg.clone());
                    continue;
                }

                let expr = exprs.get(i - offset);
                if param.is_rest {
                    let rest: Vec<Value> = args
                        .iter()
                        .skip(function.params.len() - 1)
                        .cloned()
                        .collect();

                    if expr.is_none() {
                        def_module.declare_variable(ident, Value::Vec(rest));
                    } else {
                        if let Some(_type) = param.type_annotation.as_ref() {
                            for arg in &rest {
                                if _type.is_any() {
                                    continue;
                                };

                                arg.check_type(&_type.type_name.literal(), expr.unwrap().clone())?;
                            }
                        }

                        def_module.declare_variable(ident, Value::Vec(rest));
                    }
                } else {
                    if let Some(_type) = param.type_annotation.as_ref() {
                        if arg.is_null() && _type.is_nullable {
                            def_module.declare_variable(ident, Value::Null);
                            continue;
                        }

                        if expr.is_none() {
                            return Err(MissingParameter(ident.clone(), call.span()).into());
                        }

                        if arg.is_null() && !_type.is_nullable {
                            return Err(TypeMismatch(
                                format!("Expected type {} but got null", _type.type_name.literal()),
                                expr.unwrap().clone(),
                            )
                            .into());
                        }

                        if _type.is_array {
                            match arg {
                                Value::Vec(vec) => {
                                    for arg in vec {
                                        if _type.is_any() {
                                            continue;
                                        };

                                        arg.check_type(
                                            &_type.type_name.literal(),
                                            expr.unwrap().clone(),
                                        )?;
                                    }
                                    def_module.declare_variable(ident.clone(), arg.clone());
                                }
                                _ => {
                                    return Err(TypeMismatch(
                                        format!(
                                            "Expected array type {} but got {}",
                                            _type.type_name.literal(),
                                            arg.type_name()
                                        ),
                                        expr.unwrap().clone(),
                                    )
                                    .into());
                                }
                            }
                        } else {
                            if arg.is_null() && !_type.is_nullable && !_type.is_any() {
                                arg.check_type(&_type.type_name.literal(), expr.unwrap().clone())?;
                            }
                            def_module.declare_variable(ident, arg.clone());
                        }
                    } else {
                        def_module.declare_variable(ident, arg.clone());
                    }
                }
            }
        }

        let frame = Frame::new(
            function.name.clone(),
            function.fn_token.span.clone(),
            Frame::path_or_unknown(def_module.path()),
        );
        vm.push_frame(frame);

        for stmt in function.body.stmts {
            def_module.interpret_stmt(stmt, ctx, vm)?;
        }

        vm.pop_frame();
        self.exit_scope();

        Ok(())
    }

    /// Interpret a call expression.
    ///
    /// # Arguments
    /// * `call` - [CallExpr] to interpret.
    /// * `ctx` - The context in which to interpret the call.
    ///
    /// # Returns
    /// The result of the call.
    pub fn interpret_call(
        &mut self,
        call: &CallExpr,
        ctx: &mut Context,
        vm: &mut VM,
    ) -> Result<Value> {
        log::debug!("Interpreting call");

        let args = self.interpret_possible_spread(call.args.clone(), ctx, vm)?;

        let stored_function = self
            .find_function(&call.callee)
            .ok_or_else(|| UndefinedFunctionError(call.callee.clone(), call.token.span.clone()))?
            .clone();

        match stored_function {
            StoredFunction::Native(n) => {
                self.execute_native_function(n, args, vm)?;

                Ok(vm.pop().unwrap())
            }
            StoredFunction::Function {
                function,
                defining_module,
            } => {
                let mut def_module = ctx.query_module(&defining_module).unwrap();

                match self.execute_user_defined_function(
                    function,
                    &mut def_module,
                    args,
                    ctx,
                    vm,
                    call,
                ) {
                    Ok(_) => Ok(vm.pop().unwrap_or(Value::Void)),
                    Err(e) => {
                        print_diagnostic(e, Some(def_module.source.content()));
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}
