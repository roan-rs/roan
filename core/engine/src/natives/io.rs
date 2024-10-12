use crate::{as_cast, native_function, value::Value, vm::native_fn::{NativeFunction, NativeFunctionParam}};
use crate::module::Module;

native_function!(fn __print(
    msg
) {
    let msg = as_cast!(msg, String);
    print!("{}", msg);

    Value::Void
});

native_function!(fn __eprint(
    msg
) {
    let msg = as_cast!(msg, String);
    eprint!("{}", msg);

    Value::Void
});

native_function!(fn __format(
    msg
) {
  Value::String(format!("{}", msg.to_string()))
});
