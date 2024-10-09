use crate::{module::StoredFunction, natives::io::__print};

pub mod io;

#[macro_export]
macro_rules! native_function {
    (fn $name:ident($($arg:ident: $arg_type:ident),* $(, ...$rest:ident: Vec)?) {$($body:tt)*}) => {
        pub fn $name() -> NativeFunction {
            NativeFunction {
                name: stringify!($name).to_string(),
                func: |args| {
                    let mut args_iter = args.into_iter();
                    $(
                        let $arg = match args_iter.next() {
                            Some(Value::$arg_type(value)) => value,
                            _ => panic!("Expected argument of type {} but got {:?}", stringify!($arg_type), args_iter.next().unwrap()),
                        };
                    )*

                    $(
                        let $rest = args_iter.collect::<Vec<Value>>();
                    )?

                    $($body)*
                },
                params: vec![
                    $(
                        NativeFunctionParam {
                            name: stringify!($arg).to_string(),
                            ty: stringify!(Value::$arg_type).to_string(),
                            is_rest: false,
                        },
                    )*
                    $(
                        NativeFunctionParam {
                            name: stringify!($rest).to_string(),
                            ty: "Vec<Value>".to_string(),
                            is_rest: true,
                        },
                    )?
                ],
            }
        }
    };
}

pub fn get_stored_function() -> Vec<StoredFunction> {
    vec![StoredFunction::Native(__print())]
}
