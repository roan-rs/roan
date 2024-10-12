use crate::{as_cast, native_function, value::Value, vm::native_fn::{NativeFunction, NativeFunctionParam}};
use crate::module::Module;

native_function!(
    fn __string_len(s) {
        let string = as_cast!(s, String);

        Value::Int(string.len() as i64)
    }
);

native_function!(
    fn __string_split(s, sep) {
        let s = as_cast!(s, String);
        let sep = as_cast!(sep, String);
        
        Value::Vec(
            s.split(&sep)
                .map(|s| Value::String(s.to_string()))
                .collect(),
        )
    }
);

native_function!(
    fn __string_chars(s) {
        let s = as_cast!(s, String);
        
        Value::Vec(
            s.chars().map(|c| Value::String(c.to_string())).collect(),
        )
    }
);

native_function!(
    fn __string_contains(s, needle) {
        let s = as_cast!(s, String);
        let needle = as_cast!(needle, String);

        Value::Bool(s.contains(&needle))
    }
);

native_function!(
    fn __string_starts_with(s, needle) {
        let s = as_cast!(s, String);
        let needle = as_cast!(needle, String);

        Value::Bool(s.starts_with(&needle))
    }
);

native_function!(
    fn __string_ends_with(s, needle) {
        let s = as_cast!(s, String);
        let needle = as_cast!(needle, String);

        Value::Bool(s.ends_with(&needle))
    }
);

native_function!(
    fn __string_replace(s, needle, replacement) {
        let s = as_cast!(s, String);
        let needle = as_cast!(needle, String);
        let replacement = as_cast!(replacement, String);

        Value::String(s.replace(&needle, &replacement))
    }
);

native_function!(
    fn __string_trim(s) {
        let s = as_cast!(s, String);

        Value::String(s.trim().to_string())
    }
);

native_function!(
    fn __string_trim_start(s) {
        let s = as_cast!(s, String);

        Value::String(s.trim_start().to_string())
    }
);

native_function!(
    fn __string_trim_end(s) {
        let s = as_cast!(s, String);

        Value::String(s.trim_end().to_string())
    }
);

native_function!(
    fn __string_to_uppercase(s) {
        let s = as_cast!(s, String);

        Value::String(s.to_uppercase())
    }
);

native_function!(
    fn __string_to_lowercase(s) {
        let s = as_cast!(s, String);

        Value::String(s.to_lowercase())
    }
);

native_function!(
    fn __string_reverse(s) {
        let s = as_cast!(s, String);

        Value::String(s.chars().rev().collect())
    }
);

native_function!(
    fn __string_char_at(s, index) {
        let s = as_cast!(s, String);
        let index = as_cast!(index, Int);

        let index = if index < 0 {
            s.len() as i64 + index
        } else {
            index
        };

        if index < 0 || index as usize >= s.len() {
            return Value::Null;
        }

        Value::String(s.chars().nth(index as usize).unwrap().to_string())
    }
);

native_function!(
    fn __string_char_code_at(s, index) {
        let s = as_cast!(s, String);
        let index = as_cast!(index, Int);

        let index = if index < 0 {
            s.len() as i64 + index
        } else {
            index
        };

        if index < 0 || index as usize >= s.len() {
            return Value::Null;
        }

        Value::Int(s.chars().nth(index as usize).unwrap() as i64)
    }
);

native_function!(
    fn __string_slice(s, start, end) {
        let s = as_cast!(s, String);
        let start = as_cast!(start, Int);
        let end = as_cast!(end, Int);

        let start = if start < 0 {
            s.len() as i64 + start
        } else {
            start
        };

        let end = if end < 0 {
            s.len() as i64 + end
        } else {
            end
        };

        if start < 0 || end < 0 || start as usize >= s.len() || end as usize >= s.len() {
            return Value::Null;
        }

        Value::String(s.chars().skip(start as usize).take((end - start) as usize).collect())
    }
);

native_function!(
    fn __string_index_of(s, needle) {
        let s = as_cast!(s, String);
        let needle = as_cast!(needle, String);

        Value::Int(s.find(&needle).map(|i| i as i64).unwrap_or(-1))
    }
);

native_function!(
    fn __string_last_index_of(s, needle) {
        let s = as_cast!(s, String);
        let needle = as_cast!(needle, String);

        Value::Int(s.rfind(&needle).map(|i| i as i64).unwrap_or(-1))
    }
);

// TODO: Move these to std

native_function!(
    fn __string_to_int(s) {
        let s = as_cast!(s, String);

        match s.parse::<i64>() {
            Ok(i) => Value::Int(i),
            Err(_) => Value::Null,
        }
    }
);

native_function!(
    fn __string_to_float(s) {
        let s = as_cast!(s, String);

        match s.parse::<f64>() {
            Ok(f) => Value::Float(f),
            Err(_) => Value::Null,
        }
    }
);

native_function!(
    fn __string_to_bool(s) {
        let s = as_cast!(s, String);

        match s.as_str() {
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            _ => Value::Null,
        }
    }
);

native_function!(
    fn __string_from_int(i) {
        let i = as_cast!(i, Int);

        Value::String(i.to_string())
    }
);

native_function!(
    fn __string_from_float(f) {
        let f = as_cast!(f, Float);

        Value::String(f.to_string())
    }
);

native_function!(
    fn __string_from_bool(b) {
        let b = as_cast!(b, Bool);

        Value::String(b.to_string())
    }
);

native_function!(
    fn __string_from_code(code) {
        let code = as_cast!(code, Int);

        Value::String(std::char::from_u32(code as u32).unwrap().to_string())
    }
);
