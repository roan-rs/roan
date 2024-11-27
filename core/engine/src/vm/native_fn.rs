use crate::value::Value;
use anyhow::Result;
use std::{
    fmt,
    fmt::{Display, Formatter},
};
use tracing::debug;
use roan_ast::TypeKind;

#[derive(Debug, Clone)]
pub struct NativeFunctionParam {
    pub name: String,
    pub ty: TypeKind,
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
            "Executing native function: {} with {:?} args",
            self.name,
            args.len()
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
