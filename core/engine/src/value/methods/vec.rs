use crate::{native_function, value::Value};

native_function!(
    fn __vec_len(vec: Vec) {
        Value::Int(vec.len() as i64)
    }
);
