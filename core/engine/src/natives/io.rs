use crate::native_function;
use crate::vm::value::Value;

native_function!(fn __print(
    msg: String,
    ...args: Vec
) {
    if args.is_empty() {
        print!("{}", msg);
    } else {
        print!("{}", msg);
        for arg in args {
            print!("{:?}", arg);
        }
    }

    Value::Void
});
