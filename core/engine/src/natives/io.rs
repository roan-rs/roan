use crate::{native_function, vm::value::Value};

native_function!(fn __print(
    msg: String,
    ...args: Vec
) {
    if args.is_empty() {
        print!("{}", msg);
    } else {
        let mut args_iter = args.into_iter();

        print!("{}", msg.replace("{}", &args_iter.next().unwrap().to_string()));
    }

    Value::Void
});
