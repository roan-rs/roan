use crate::{
    native_function,
    value::Value,
    vm::native_fn::{NativeFunction, NativeFunctionParam},
};

native_function!(
    fn __string_len(s: String) {
        Value::Int(s.len() as i64)
    }
);

native_function!(
    fn __string_split(s: String, sep: String) {
        Value::Vec(
            s.split(&sep)
                .map(|s| Value::String(s.to_string()))
                .collect(),
        )
    }
);
