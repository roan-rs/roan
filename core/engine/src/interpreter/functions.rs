use crate::{
    context::Context,
    module::{Module, StoredFunction},
    value::Value,
    vm::{native_fn::NativeFunction, VM},
};
use anyhow::Result;
use roan_ast::CallExpr;
use roan_error::{error::RoanError::UndefinedFunctionError, frame::Frame, print_diagnostic};
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
        _: &CallExpr,
    ) -> Result<()> {
        debug!("Executing user-defined function: {}", function.name);

        self.enter_scope();

        let mut offset = 0;

        // TODO: handle rest parameters
        for (param, arg) in function
            .params
            .iter()
            .zip(args.iter().chain(std::iter::repeat(&Value::Null)))
        {
            let ident = param.ident.literal();
            // Maybe we could find a better way to handle this
            if ident == "self" {
                offset += 1;

                def_module.declare_variable(ident, arg.clone());
                continue;
            }

            if param.is_rest {
                let rest: Vec<Value> = args
                    .iter()
                    .skip(function.params.len() - 1)
                    .cloned()
                    .collect();

                let _type = param.type_annotation.clone();

                def_module.declare_variable(ident, Value::Vec(rest));
            } else {
                def_module.declare_variable(ident, arg.clone());
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
                        print_diagnostic(&e, Some(def_module.source.content()), def_module.path());
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}
