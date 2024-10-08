use roan_ast::FnParam;
use std::{
    fmt,
    fmt::{Display, Formatter},
};
use log::debug;
use crate::value::Value;

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

    pub fn call(&mut self, args: Vec<Value>) -> anyhow::Result<Value> {
        debug!("Executing native function: {}", self.name);

        let mut params = vec![];
        for (param, val) in self.params.iter().zip(args.clone()) {
            if param.is_rest {
                let rest = args.iter().skip(self.params.len() - 1).cloned().collect();

                params.push(Value::Vec(rest));
            } else {
                params.push(val);
            }
        }

        Ok((self.func)(params))
    }
}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn {}>", self.name)
    }
}
