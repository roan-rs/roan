use crate::{
    native_function,
    value::Value,
    vm::native_fn::{NativeFunction, NativeFunctionParam},
};

native_function!(fn __print(
    msg: String,
    ...args: Vec
) {
    if args.is_empty() {
        print!("{}", msg);
    } else {
        print!("{}", format(msg, args));
    }

    Value::Void
});

fn format(msg: String, args: Vec<Value>) -> String {
    if args.is_empty() {
        return msg;
    }

    let args_iter = args.into_iter();
    let mut formatted = msg.to_string();

    for arg in args_iter {
        formatted = formatted.replace("{}", &arg.to_string());
    }

    formatted
}