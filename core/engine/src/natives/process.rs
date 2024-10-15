use crate::{
    as_cast, native_function,
    value::Value,
    vm::native_fn::{NativeFunction, NativeFunctionParam},
};

native_function!(
    fn __exit(status) {
        let status = as_cast!(status, Int);
        std::process::exit(status as i32);
    }
);

native_function!(
    fn __abort() {
        std::process::abort();
    }
);

native_function!(
    fn __pid() {
        Value::Int(std::process::id() as i64)
    }
);
