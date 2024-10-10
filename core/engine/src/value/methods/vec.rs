use crate::{
    native_function,
    value::Value,
    vm::native_fn::{NativeFunction, NativeFunctionParam},
};

native_function!(
    fn __vec_len(vec: Vec) {
        Value::Int(vec.len() as i64)
    }
);
