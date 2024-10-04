pub mod io;

/// Macro for defining a native function
#[macro_export]
macro_rules! native_function {
    (fn $name:ident ($($arg:ident: $arg_type:ident),*) {$($body:tt)*}) => {
        use crate::vm::native_fn::{NativeFunctionParam,NativeFunction};
        use crate::vm::value::Value;

        pub fn $name() -> NativeFunction {
            NativeFunction {
                name: stringify!($name).to_string(),
                func: |args| {
                    let mut args_iter = args.into_iter();
                    $(
                        let $arg = match args_iter.next() {
                            Some(Value::$arg_type(value)) => value,
                            _ => panic!("Expected argument of type {}", stringify!($arg_type)),
                        };
                    )*

                    $($body)*
                },
                params: vec![
                    $(
                        NativeFunctionParam {
                            name: stringify!($arg).to_string(),
                            ty: stringify!(Value::$arg_type).to_string(),
                            is_rest: stringify!($arg).starts_with("..."),
                        },
                    )*
                ],
            }
        }
    };
}
