use crate::{
    as_cast, native_function,
    value::Value,
    vm::native_fn::{NativeFunction, NativeFunctionParam},
};

native_function!(
    fn __char_is_alphabetic(c) {
        let c = as_cast!(c, Char);

        Value::Bool(c.is_alphabetic())
    }
);

native_function!(
    fn __char_is_alphanumeric(c) {
        let c = as_cast!(c, Char);

        Value::Bool(c.is_alphanumeric())
    }
);

native_function!(
    fn __char_is_ascii(c) {
        let c = as_cast!(c, Char);

        Value::Bool(c.is_ascii())
    }
);

native_function!(
    fn __char_is_ascii_alphabetic(c) {
        let c = as_cast!(c, Char);

        Value::Bool(c.is_ascii_alphabetic())
    }
);

native_function!(
    fn __char_is_ascii_alphanumeric(c) {
        let c = as_cast!(c, Char);

        Value::Bool(c.is_ascii_alphanumeric())
    }
);

native_function!(
    fn __char_is_ascii_control(c) {
        let c = as_cast!(c, Char);

        Value::Bool(c.is_ascii_control())
    }
);

native_function!(
    fn __char_is_ascii_digit(c) {
        let c = as_cast!(c, Char);

        Value::Bool(c.is_ascii_digit())
    }
);

native_function!(
    fn __char_is_ascii_graphic(c) {
        let c = as_cast!(c, Char);

        Value::Bool(c.is_ascii_graphic())
    }
);

native_function!(
    fn __char_is_ascii_lowercase(c) {
        let c = as_cast!(c, Char);

        Value::Bool(c.is_ascii_lowercase())
    }
);

native_function!(
    fn __char_is_ascii_punctuation(c) {
        let c = as_cast!(c, Char);

        Value::Bool(c.is_ascii_punctuation())
    }
);

native_function!(
    fn __char_is_ascii_uppercase(c) {
        let c = as_cast!(c, Char);

        Value::Bool(c.is_ascii_uppercase())
    }
);

native_function!(
    fn __char_is_ascii_whitespace(c) {
        let c = as_cast!(c, Char);

        Value::Bool(c.is_ascii_whitespace())
    }
);

native_function!(
    fn __char_is_control(c) {
        let c = as_cast!(c, Char);

        Value::Bool(c.is_control())
    }
);

native_function!(
    fn __char_is_digit(c) {
        let c = as_cast!(c, Char);

        Value::Bool(c.is_digit(10))
    }
);

native_function!(
    fn __char_is_lowercase(c) {
        let c = as_cast!(c, Char);

        Value::Bool(c.is_lowercase())
    }
);

native_function!(
    fn __char_is_numeric(c) {
        let c = as_cast!(c, Char);

        Value::Bool(c.is_numeric())
    }
);

native_function!(
    fn __char_is_uppercase(c) {
        let c = as_cast!(c, Char);

        Value::Bool(c.is_uppercase())
    }
);

native_function!(
    fn __char_is_whitespace(c) {
        let c = as_cast!(c, Char);

        Value::Bool(c.is_whitespace())
    }
);

native_function!(
    fn __char_to_ascii_lowercase(c) {
        let c = as_cast!(c, Char);

        Value::Char(c.to_ascii_lowercase())
    }
);

native_function!(
    fn __char_to_ascii_uppercase(c) {
        let c = as_cast!(c, Char);

        Value::Char(c.to_ascii_uppercase())
    }
);

native_function!(
    fn __char_to_lowercase(c) {
        let c = as_cast!(c, Char);

        Value::Char(c.to_lowercase().next().unwrap())
    }
);

native_function!(
    fn __char_to_uppercase(c) {
        let c = as_cast!(c, Char);

        Value::Char(c.to_uppercase().next().unwrap())
    }
);

native_function!(
    fn __char_is_digit_in_base(c, base) {
        let c = as_cast!(c, Char);
        let base = as_cast!(base, Int);

        Value::Bool(c.is_digit(base as u32))
    }
);

native_function!(
    fn __char_escape_default(c) {
        let c = as_cast!(c, Char);

        Value::String(c.escape_default().to_string().into())
    }
);

native_function!(
    fn __char_escape_unicode(c) {
        let c = as_cast!(c, Char);

        Value::String(c.escape_unicode().to_string().into())
    }
);

native_function!(
    fn __char_from_digit(digit, base) {
        let digit = as_cast!(digit, Int);
        let base = as_cast!(base, Int);

        match char::from_digit(digit as u32, base as u32) {
            Some(c) => Value::Char(c),
            None => Value::Null
        }
    }
);

native_function!(
    fn __char_len_utf8(c) {
        let c = as_cast!(c, Char);

        Value::Int(c.len_utf8() as i64)
    }
);

native_function!(
    fn __char_to_string(c) {
        let c = as_cast!(c, Char);

        Value::String(c.to_string().into())
    }
);
