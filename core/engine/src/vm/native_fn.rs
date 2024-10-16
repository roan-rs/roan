use crate::{context::Context, module::Module, value::Value, vm::VM};
use anyhow::Result;
use log::debug;
use roan_error::frame::Frame;
use std::{
    fmt,
    fmt::{Display, Formatter},
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone)]
pub struct NativeFunctionParam {
    pub name: String,
    pub ty: String,
    pub is_rest: bool,
}

#[derive(Debug, Clone)]
pub struct NativeFunction {
    pub name: String,
    pub func: fn(args: Vec<Value>) -> Value,
    pub params: Vec<NativeFunctionParam>,
}

impl NativeFunction {
    pub fn new(
        name: impl Into<String>,
        params: Vec<NativeFunctionParam>,
        func: fn(args: Vec<Value>) -> Value,
    ) -> Self {
        Self {
            name: name.into(),
            func,
            params,
        }
    }

    pub fn call(&mut self, args: Vec<Value>) -> Result<Value> {
        debug!(
            "Executing native function: {} with args {:?}",
            self.name, args
        );

        let mut params = vec![];
        for (param, val) in self.params.iter().zip(args.clone()) {
            if param.is_rest {
                let rest: Vec<Value> = args.iter().skip(self.params.len() - 1).cloned().collect();

                params.extend(rest);
            } else {
                params.push(val);
            }
        }

        Ok((self.func)(params))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{as_cast, native_function, value::Value};

    native_function!(fn __add_str(str1, str2) {
        let str1 = as_cast!(str1, String);
        let str2 = as_cast!(str2, String);

        Value::String(format!("{}{}", str1, str2))
    });

    #[test]
    fn test_native_function() {
        assert_eq!(
            __add_str()
                .call(vec![
                    Value::String("Hello".to_string()),
                    Value::String("World".to_string())
                ])
                .unwrap(),
            Value::String("HelloWorld".to_string())
        );
    }

    native_function!(fn __test1(a, b) {
        assert_eq!(a, Value::Int(1));
        assert_eq!(b, Value::Int(2));
        Value::Null
    });

    #[test]
    fn test_native_function_with_params() {
        assert_eq!(
            __test1().call(vec![Value::Int(1), Value::Int(2)]).unwrap(),
            Value::Null
        );
    }

    native_function!(fn __test(a, b, ...rest) {
        assert_eq!(a, Value::Int(1));
        assert_eq!(b, Value::Int(2));
        assert_eq!(rest[0], Value::Int(3));
        Value::Null
    });

    #[test]
    fn test_native_function_with_rest_param() {
        assert_eq!(
            __test()
                .call(vec![
                    Value::Int(1),
                    Value::Int(2),
                    Value::Int(3),
                    Value::Int(4)
                ])
                .unwrap(),
            Value::Null
        );
    }
}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn {}>", self.name)
    }
}

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
        defining_module: Arc<Mutex<Module>>,
        args: Vec<Value>,
        ctx: &Context,
        vm: &mut VM,
    ) -> Result<()> {
        debug!("Executing user-defined function: {}", function.name);

        self.enter_scope();

        {
            let mut defining_module_guard = defining_module.lock().unwrap();

            for (param, arg) in function
                .params
                .iter()
                .zip(args.iter().chain(std::iter::repeat(&Value::Null)))
            {
                let ident = param.ident.literal();
                if param.is_rest {
                    let rest = args
                        .iter()
                        .skip(function.params.len() - 1)
                        .cloned()
                        .collect();
                    defining_module_guard.declare_variable(ident, Value::Vec(rest));
                } else {
                    defining_module_guard.declare_variable(ident, arg.clone());
                }
            }
        }

        let frame = Frame::new(
            function.name.clone(),
            function.fn_token.span.clone(),
            Frame::path_or_unknown(defining_module.lock().unwrap().path()),
        );
        vm.push_frame(frame);

        {
            let mut defining_module_guard = defining_module.lock().unwrap();
            for stmt in function.body.stmts {
                defining_module_guard.interpret_stmt(stmt, ctx, vm)?;
            }
        }

        vm.pop_frame();
        self.exit_scope();

        Ok(())
    }
}
