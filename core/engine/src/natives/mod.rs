use crate::{
    module::StoredFunction,
    natives::{
        debug::{__eprint, __format, __print},
        process::{__abort, __exit, __pid},
    },
    value::Value,
    vm::native_fn::{NativeFunction, NativeFunctionParam},
};
use std::{panic, panic::panic_any};

pub mod debug;
mod process;

#[macro_export]
macro_rules! native_function {
    (fn $name:ident($($arg:ident),* $(, ...$rest:ident)?) {$($body:tt)*}) => {
        #[allow(unused_mut, unused_variables)]
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

native_function!(
    fn type_of(value) {
        Value::String(value.type_name())
    }
);

native_function!(
    fn __panic(msg) {
        let msg = as_cast!(msg, String);

        // Save panic hook to restore it later
        let old_hook = std::panic::take_hook();

        panic::set_hook(Box::new(|panic_info| {
            let payload = panic_info.payload().downcast_ref::<String>().unwrap();
            eprintln!("program panicked");
            eprintln!("{}", payload);
        }));

        panic_any(msg);

        // Restore the original hook afterward if needed.
        panic::set_hook(old_hook);

        Value::Void
    }
);

pub fn get_stored_function() -> Vec<StoredFunction> {
    vec![
        __print(),
        __format(),
        __eprint(),
        __exit(),
        __abort(),
        __pid(),
        type_of(),
        __panic(),
    ]
    .into_iter()
    .map(|f| StoredFunction::Native(f))
    .collect()
}
