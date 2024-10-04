use crate::native_function;
use crate::vm::value::Value;

native_function!(fn __print(
    msg: String,
    ...args: Vec
) {
    for arg in args {
        print!("{:?}", arg);
    }

    Value::Void
});