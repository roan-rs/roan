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

native_function!(fn __format(
    msg: String,
    ...args: Vec
) {
  Value::String(format(msg, args))
});

fn format(msg: String, args: Vec<Value>) -> String {
    let mut formatted = String::new();
    let mut arg_iter = args.iter().map(|v| v.to_string()).peekable();
    let mut chars = msg.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'}') {
            chars.next(); // Consume '}'
            if let Some(arg) = arg_iter.next() {
                formatted.push_str(&arg);
            } else {
                formatted.push_str("{}"); // No corresponding argument
            }
        } else {
            formatted.push(c);
        }
    }

    formatted
}
