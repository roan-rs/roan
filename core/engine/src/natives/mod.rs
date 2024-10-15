use crate::{
    module::StoredFunction,
    natives::io::{__eprint, __format, __print},
};
use crate::natives::process::__exit;

pub mod io;
mod process;

#[macro_export]
macro_rules! native_function {
    (fn $name:ident($($arg:ident),* $(, ...$rest:ident)?) {$($body:tt)*}) => {
        pub fn $name() -> NativeFunction {
            NativeFunction {
                name: stringify!($name).to_string(),
                func: |args| {
                    let mut args_iter = args.into_iter();
                    $(
                        let $arg = match args_iter.next() {
                            Some(value) => value,
                            None => panic!("Expected argument but got None"),
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
                            ty: "Value".to_string(),
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

#[macro_export]
macro_rules! as_cast {
    ($val:expr, $ty:ident) => {
        match $val {
            Value::$ty(val) => val,
            // TODO: Replace with throw! macro
            _ => panic!("Expected {} but got {:?}", stringify!($ty), $val),
        }
    };
}

pub fn get_stored_function() -> Vec<StoredFunction> {
    vec![__print(), __format(), __eprint(), __exit()]
        .into_iter()
        .map(|f| StoredFunction::Native(f))
        .collect()
}
