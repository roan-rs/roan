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

native_function!(
    fn __char_to_int(c) {
        let c = as_cast!(c, Char);

        Value::Int(c as i64)
    }
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_char_is_alphabetic() {
        let result = __char_is_alphabetic().call(vec![Value::Char('a')]).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_is_alphanumeric() {
        let result = __char_is_alphanumeric()
            .call(vec![Value::Char('a')])
            .unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_is_ascii() {
        let result = __char_is_ascii().call(vec![Value::Char('a')]).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_is_ascii_alphabetic() {
        let result = __char_is_ascii_alphabetic()
            .call(vec![Value::Char('a')])
            .unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_is_ascii_alphanumeric() {
        let result = __char_is_ascii_alphanumeric()
            .call(vec![Value::Char('a')])
            .unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_is_ascii_control() {
        let result = __char_is_ascii_control()
            .call(vec![Value::Char('\n')])
            .unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_is_ascii_digit() {
        let result = __char_is_ascii_digit()
            .call(vec![Value::Char('1')])
            .unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_is_ascii_graphic() {
        let result = __char_is_ascii_graphic()
            .call(vec![Value::Char('a')])
            .unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_is_ascii_lowercase() {
        let result = __char_is_ascii_lowercase()
            .call(vec![Value::Char('a')])
            .unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_is_ascii_punctuation() {
        let result = __char_is_ascii_punctuation()
            .call(vec![Value::Char('!')])
            .unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_is_ascii_uppercase() {
        let result = __char_is_ascii_uppercase()
            .call(vec![Value::Char('A')])
            .unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_is_ascii_whitespace() {
        let result = __char_is_ascii_whitespace()
            .call(vec![Value::Char(' ')])
            .unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_is_control() {
        let result = __char_is_control().call(vec![Value::Char('\n')]).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_is_digit() {
        let result = __char_is_digit().call(vec![Value::Char('1')]).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_is_lowercase() {
        let result = __char_is_lowercase().call(vec![Value::Char('a')]).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_is_numeric() {
        let result = __char_is_numeric().call(vec![Value::Char('1')]).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_is_uppercase() {
        let result = __char_is_uppercase().call(vec![Value::Char('A')]).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_is_whitespace() {
        let result = __char_is_whitespace().call(vec![Value::Char(' ')]).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_to_ascii_lowercase() {
        let result = __char_to_ascii_lowercase()
            .call(vec![Value::Char('A')])
            .unwrap();

        assert_eq!(result, Value::Char('a'));
    }

    #[test]
    fn test_char_to_ascii_uppercase() {
        let result = __char_to_ascii_uppercase()
            .call(vec![Value::Char('a')])
            .unwrap();

        assert_eq!(result, Value::Char('A'));
    }

    #[test]
    fn test_char_to_lowercase() {
        let result = __char_to_lowercase().call(vec![Value::Char('A')]).unwrap();

        assert_eq!(result, Value::Char('a'));
    }

    #[test]
    fn test_char_to_uppercase() {
        let result = __char_to_uppercase().call(vec![Value::Char('a')]).unwrap();

        assert_eq!(result, Value::Char('A'));
    }

    #[test]
    fn test_char_is_digit_in_base() {
        let result = __char_is_digit_in_base()
            .call(vec![Value::Char('1'), Value::Int(10)])
            .unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_escape_default() {
        let result = __char_escape_default()
            .call(vec![Value::Char('\n')])
            .unwrap();

        assert_eq!(result, Value::String("\\n".into()));
    }

    #[test]
    fn test_char_escape_unicode() {
        let result = __char_escape_unicode()
            .call(vec![Value::Char('a')])
            .unwrap();

        assert_eq!(result, Value::String("\\u{61}".into()));
    }

    #[test]
    fn test_char_from_digit() {
        let result = __char_from_digit()
            .call(vec![Value::Int(1), Value::Int(10)])
            .unwrap();

        assert_eq!(result, Value::Char('1'));
    }

    #[test]
    fn test_char_len_utf8() {
        let result = __char_len_utf8().call(vec![Value::Char('a')]).unwrap();

        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_char_is_ascii_non_ascii() {
        let result = __char_is_ascii().call(vec![Value::Char('é')]).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_char_is_ascii_lowercase_non_ascii() {
        let result = __char_is_ascii_lowercase()
            .call(vec![Value::Char('é')])
            .unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_char_is_ascii_uppercase_non_ascii() {
        let result = __char_is_ascii_uppercase()
            .call(vec![Value::Char('É')])
            .unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_char_from_digit_invalid() {
        let result = __char_from_digit()
            .call(vec![Value::Int(16), Value::Int(10)]) // 16 is invalid in base 10
            .unwrap();

        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_char_is_digit_in_base_invalid_base() {
        let result = __char_is_digit_in_base()
            .call(vec![Value::Char('A'), Value::Int(10)]) // 'A' is not a digit in base 10
            .unwrap();

        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_char_len_utf8_emoji() {
        let result = __char_len_utf8().call(vec![Value::Char('😊')]).unwrap();
        assert_eq!(result, Value::Int(4)); // UTF-8 length of emoji
    }
}
