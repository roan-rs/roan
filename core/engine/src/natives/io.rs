use crate::native_function;

native_function!(fn __print(
    arg: String
) {
    println!("dwa {}", arg);

    Value::Void
});