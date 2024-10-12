use crate::{as_cast, native_function, value::Value, vm::native_fn::{NativeFunction, NativeFunctionParam}};
use crate::module::Module;

native_function!(
    fn __string_len(s) {
        let string = as_cast!(s, String);

        Value::Int(string.len() as i64)
    }
);

native_function!(
    fn __string_split(s, sep) {
        let s = as_cast!(s, String);
        let sep = as_cast!(sep, String);
        
        Value::Vec(
            s.split(&sep)
                .map(|s| Value::String(s.to_string()))
                .collect(),
        )
    }
);

native_function!(
    fn __string_chars(s) {
        let s = as_cast!(s, String);
        
        Value::Vec(
            s.chars().map(|c| Value::String(c.to_string())).collect(),
        )
    }
);