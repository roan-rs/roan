use crate::{
    as_cast,
    native_function,
    value::Value,
    vm::native_fn::{NativeFunction, NativeFunctionParam},
};

native_function!(
    fn __vec_len(vec) {
        let vec = as_cast!(vec, Vec);

        Value::Int(vec.len() as i64)
    }
);

native_function!(
    fn __vec_next(vec) {
        let mut vec = as_cast!(vec, Vec);

        vec = vec.into_iter().skip(1).collect();

        Value::Vec(vec)
    }
);
