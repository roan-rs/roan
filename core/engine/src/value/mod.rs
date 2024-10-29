use crate::{
    entries,
    module::StoredStruct,
    value::methods::{
        char::{
            __char_escape_default, __char_escape_unicode, __char_from_digit, __char_is_alphabetic,
            __char_is_alphanumeric, __char_is_ascii, __char_is_ascii_alphabetic,
            __char_is_ascii_alphanumeric, __char_is_ascii_control, __char_is_ascii_digit,
            __char_is_ascii_graphic, __char_is_ascii_lowercase, __char_is_ascii_punctuation,
            __char_is_ascii_uppercase, __char_is_ascii_whitespace, __char_is_control,
            __char_is_digit, __char_is_digit_in_base, __char_is_lowercase, __char_is_numeric,
            __char_is_uppercase, __char_is_whitespace, __char_len_utf8, __char_to_ascii_lowercase,
            __char_to_ascii_uppercase, __char_to_int, __char_to_lowercase, __char_to_string,
            __char_to_uppercase,
        },
        string::{
            __string_char_at, __string_char_code_at, __string_chars, __string_contains,
            __string_ends_with, __string_index_of, __string_last_index_of, __string_len,
            __string_replace, __string_reverse, __string_slice, __string_split,
            __string_starts_with, __string_to_lowercase, __string_to_uppercase, __string_trim,
            __string_trim_end, __string_trim_start,
        },
        vec::{__vec_len, __vec_next},
    },
    vm::native_fn::NativeFunction,
};
use anyhow::Result;
use roan_ast::{Literal, LiteralType};
use roan_error::{error::PulseError::TypeMismatch, TextSpan};
use std::{
    collections::HashMap,
    fmt::{write, Debug, Display},
    ops,
};
use indexmap::IndexMap;

pub mod methods {
    pub mod char;
    pub mod string;
    pub mod vec;
}

#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Vec(Vec<Value>),
    Struct(StoredStruct, HashMap<String, Value>),
    Object(IndexMap<String, Value>),
    Null,
    Void,
}

impl Value {
    pub fn builtin_methods(&self) -> HashMap<String, NativeFunction> {
        match self {
            Value::Vec(_) => {
                entries!(
                    "len" => __vec_len(),
                    "next" => __vec_next()
                )
            }
            Value::String(_) => {
                entries!(
                    "len" => __string_len(),
                    "split" => __string_split(),
                    "chars" => __string_chars(),
                    "contains" => __string_contains(),
                    "starts_with" => __string_starts_with(),
                    "ends_with" => __string_ends_with(),
                    "replace" => __string_replace(),
                    "trim" => __string_trim(),
                    "trim_start" => __string_trim_start(),
                    "trim_end" => __string_trim_end(),
                    "to_uppercase" => __string_to_uppercase(),
                    "to_lowercase" => __string_to_lowercase(),
                    "reverse" => __string_reverse(),
                    "char_at" => __string_char_at(),
                    "char_code_at" => __string_char_code_at(),
                    "slice" => __string_slice(),
                    "index_of" => __string_index_of(),
                    "last_index_of" => __string_last_index_of()
                )
            }
            Value::Char(_) => {
                entries!(
                    "is_alphabetic" => __char_is_alphabetic(),
                    "is_alphanumeric" => __char_is_alphanumeric(),
                    "is_ascii" => __char_is_ascii(),
                    "is_ascii_alphabetic" => __char_is_ascii_alphabetic(),
                    "is_ascii_alphanumeric" => __char_is_ascii_alphanumeric(),
                    "is_ascii_control" => __char_is_ascii_control(),
                    "is_ascii_digit" => __char_is_ascii_digit(),
                    "is_ascii_graphic" => __char_is_ascii_graphic(),
                    "is_ascii_lowercase" => __char_is_ascii_lowercase(),
                    "is_ascii_punctuation" => __char_is_ascii_punctuation(),
                    "is_ascii_uppercase" => __char_is_ascii_uppercase(),
                    "is_ascii_whitespace" => __char_is_ascii_whitespace(),
                    "is_control" => __char_is_control(),
                    "is_digit" => __char_is_digit(),
                    "is_lowercase" => __char_is_lowercase(),
                    "is_numeric" => __char_is_numeric(),
                    "is_uppercase" => __char_is_uppercase(),
                    "is_whitespace" => __char_is_whitespace(),
                    "to_ascii_lowercase" => __char_to_ascii_lowercase(),
                    "to_ascii_uppercase" => __char_to_ascii_uppercase(),
                    "to_lowercase" => __char_to_lowercase(),
                    "to_uppercase" => __char_to_uppercase(),
                    "is_digit_in_base" => __char_is_digit_in_base(),
                    "escape_default" => __char_escape_default(),
                    "escape_unicode" => __char_escape_unicode(),
                    "from_digit" => __char_from_digit(),
                    "len_utf8" => __char_len_utf8(),
                    "to_string" => __char_to_string(),
                    "to_int" => __char_to_int()
                )
            }
            _ => HashMap::new(),
        }
    }
}

impl Value {
    pub fn from_literal(literal: Literal) -> Self {
        match literal.value {
            LiteralType::Int(i) => Value::Int(i),
            LiteralType::Float(f) => Value::Float(f),
            LiteralType::Bool(b) => Value::Bool(b),
            LiteralType::String(s) => Value::String(s.clone()),
            LiteralType::Null => Value::Null,
            LiteralType::Char(c) => Value::Char(c),
        }
    }
}

impl ops::Add for Value {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self.clone(), other.clone()) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
            (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 + b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a + b as f64),
            (Value::String(a), Value::String(b)) => Value::String(a + &b),
            (Value::Char(a), Value::Char(b)) => Value::String(format!("{}{}", a, b)),
            (Value::Char(a), Value::String(b)) => Value::String(format!("{}{}", a, b)),
            (Value::String(a), Value::Char(b)) => Value::String(format!("{}{}", a, b)),
            _ => panic!(
                "Cannot add values of different types: {:?} and {:?}",
                self, other
            ),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "Int({})", i),
            Value::Float(fl) => write!(f, "Float({})", fl),
            Value::Bool(b) => write!(f, "Bool({})", b),
            Value::String(s) => write!(f, "String({})", s),
            Value::Vec(v) => write!(f, "Vec({:?})", v),
            Value::Null => write!(f, "Null"),
            Value::Void => write!(f, "Void"),
            Value::Struct(struct_def, fields) => {
                write!(f, "Struct({} with fields: ", struct_def.name.literal())?;
                for (name, val) in fields {
                    write!(f, "{}: {:?}, ", name, val)?;
                }
                write!(f, ")")
            }
            Value::Char(c) => write!(f, "Char({})", c),
            Value::Object(fields) => {
                write!(f, "{:#?}", fields)
            }
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::Vec(v) => {
                write!(f, "[")?;
                for (i, val) in v.iter().enumerate() {
                    write!(f, "{}", val)?;
                    if i < v.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Value::Null => write!(f, "null"),
            Value::Void => write!(f, "void"),
            Value::Struct(st, fields) => {
                let def = st.clone();

                write!(f, "{} {{", def.name.literal())?;
                for (i, (name, val)) in fields.iter().enumerate() {
                    write!(f, "{}: {}", name, val)?;
                    if i < fields.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "}}")
            }
            Value::Char(c) => write!(f, "{}", c),
            Value::Object(fields) => {
                write!(f, "{{")?;
                for (i, (name, val)) in fields.iter().enumerate() {
                    write!(f, "{}: {}", name, val)?;
                    if i < fields.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "}}")
            }
        }
    }
}

impl ops::Sub for Value {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
            (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 - b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a - b as f64),
            _ => panic!("Cannot subtract values of different types"),
        }
    }
}

impl ops::Mul for Value {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
            (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 * b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a * b as f64),
            _ => panic!("Cannot multiply values of different types"),
        }
    }
}

impl ops::Div for Value {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a / b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
            (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 / b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a / b as f64),
            _ => panic!("Cannot divide values of different types"),
        }
    }
}

impl ops::Rem for Value {
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a % b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a % b),
            (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 % b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a % b as f64),
            _ => panic!("Cannot modulo values of different types"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Vec(a), Value::Vec(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for (a, b) in a.iter().zip(b.iter()) {
                    if a != b {
                        return false;
                    }
                }
                true
            }
            (Value::Null, Value::Null) => true,
            (Value::Void, Value::Void) => true,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::Char(a), Value::String(b)) => a.to_string() == *b,
            (Value::String(a), Value::Char(b)) => a == &b.to_string(),
            _ => false,
        }
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b),
            (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)),
            _ => None,
        }
    }
}

impl Value {
    pub fn pow(self, other: Self) -> Self {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a.pow(b as u32)),
            (Value::Float(a), Value::Float(b)) => Value::Float(a.powf(b)),
            (Value::Int(a), Value::Float(b)) => Value::Float((a as f64).powf(b)),
            (Value::Float(a), Value::Int(b)) => Value::Float(a.powf(b as f64)),
            _ => panic!("Cannot apply power operator on values of different or unsupported types"),
        }
    }
}

impl Value {
    pub fn access_index(&self, index: Self) -> Self {
        match self {
            Value::Vec(v) => match index {
                Value::Int(i) => v.get(i as usize).cloned().unwrap_or(Value::Null),
                _ => Value::Null,
            },
            Value::String(s) => match index {
                Value::Int(i) => {
                    if i < 0 {
                        Value::Null
                    } else {
                        s.chars()
                            .nth(i as usize)
                            .map(Value::Char)
                            .unwrap_or(Value::Null)
                    }
                }
                _ => Value::Null,
            },
            Value::Object(fields) => match index {
                Value::String(key) => fields.get(&key).cloned().unwrap_or(Value::Null),
                _ => Value::Null,
            },
            // TODO: proper error handling
            _ => panic!("Cannot access index of non-indexable value"),
        }
    }
}

impl Value {
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Vec(_))
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Value::Int(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, Value::Struct(_, _))
    }

    pub fn is_void(&self) -> bool {
        matches!(self, Value::Void)
    }
}

impl Value {
    pub fn check_type(&self, expected_type: &str, span: TextSpan) -> Result<()> {
        if self.is_type(expected_type) {
            Ok(())
        } else {
            Err(TypeMismatch(
                format!(
                    "Expected type {} but got {}",
                    expected_type,
                    self.type_name()
                ),
                span,
            )
            .into())
        }
    }

    pub fn is_type(&self, type_name: &str) -> bool {
        match type_name {
            "int" => self.is_int(),
            "float" => self.is_float(),
            "bool" => self.is_bool(),
            "string" => self.is_string(),
            "null" => self.is_null(),
            "void" => self.is_void(),
            _ => false,
        }
    }

    pub fn type_name(&self) -> String {
        match self {
            Value::Int(_) => "int".to_string(),
            Value::Float(_) => "float".to_string(),
            Value::Bool(_) => "bool".to_string(),
            Value::String(_) => "string".to_string(),
            // Type of vector is based on the type of its first element
            Value::Vec(vals) => {
                if vals.is_empty() {
                    "void[]".to_string()
                } else {
                    format!("{}[]", vals[0].type_name())
                }
            }
            Value::Struct(struct_def, _) => struct_def.name.literal(),
            Value::Null => "null".to_string(),
            Value::Void => "void".to_string(),
            Value::Char(_) => "char".to_string(),
            Value::Object(_) => "object".to_string(),
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::Bool(b) => *b,
            Value::String(s) => !s.is_empty(),
            Value::Vec(v) => !v.is_empty(),
            Value::Null => false,
            Value::Void => false,
            Value::Struct(_, _) => true,
            Value::Char(_) => true,
            Value::Object(_) => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_add() {
        assert_eq!(Value::Int(1) + Value::Int(2), Value::Int(3));
        assert_eq!(Value::Float(1.0) + Value::Float(2.0), Value::Float(3.0));
        assert_eq!(Value::Int(1) + Value::Float(2.0), Value::Float(3.0));
        assert_eq!(Value::Float(1.0) + Value::Int(2), Value::Float(3.0));
        assert_eq!(
            Value::String("Hello".to_string()) + Value::String("World".to_string()),
            Value::String("HelloWorld".to_string())
        );
    }

    #[test]
    fn test_value_sub() {
        assert_eq!(Value::Int(1) - Value::Int(2), Value::Int(-1));
        assert_eq!(Value::Float(1.0) - Value::Float(2.0), Value::Float(-1.0));
        assert_eq!(Value::Int(1) - Value::Float(2.0), Value::Float(-1.0));
        assert_eq!(Value::Float(1.0) - Value::Int(2), Value::Float(-1.0));
    }

    #[test]
    fn test_value_mul() {
        assert_eq!(Value::Int(1) * Value::Int(2), Value::Int(2));
        assert_eq!(Value::Float(1.0) * Value::Float(2.0), Value::Float(2.0));
        assert_eq!(Value::Int(1) * Value::Float(2.0), Value::Float(2.0));
        assert_eq!(Value::Float(1.0) * Value::Int(2), Value::Float(2.0));
    }

    #[test]
    fn test_value_div() {
        assert_eq!(Value::Int(1) / Value::Int(2), Value::Int(0));
        assert_eq!(Value::Float(1.0) / Value::Float(2.0), Value::Float(0.5));
        assert_eq!(Value::Int(1) / Value::Float(2.0), Value::Float(0.5));
        assert_eq!(Value::Float(5.5) / Value::Int(2), Value::Float(2.75));
    }

    #[test]
    fn test_value_rem() {
        assert_eq!(Value::Int(1) % Value::Int(2), Value::Int(1));
        assert_eq!(Value::Float(1.0) % Value::Float(2.0), Value::Float(1.0));
        assert_eq!(Value::Int(1) % Value::Float(2.0), Value::Float(1.0));
        assert_eq!(Value::Float(5.5) % Value::Int(2), Value::Float(1.5));
    }

    #[test]
    fn test_value_pow() {
        assert_eq!(Value::Int(2).pow(Value::Int(3)), Value::Int(8));
        assert_eq!(Value::Float(2.0).pow(Value::Float(3.0)), Value::Float(8.0));
        assert_eq!(Value::Int(2).pow(Value::Float(3.0)), Value::Float(8.0));
        assert_eq!(Value::Float(2.0).pow(Value::Int(3)), Value::Float(8.0));
    }

    #[test]
    fn test_value_access_index() {
        assert_eq!(
            Value::Vec(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
                .access_index(Value::Int(1)),
            Value::Int(2)
        );
        assert_eq!(
            Value::Vec(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
                .access_index(Value::Int(3)),
            Value::Null
        );
        assert_eq!(
            Value::Vec(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
                .access_index(Value::Float(1.0)),
            Value::Null
        );
        assert_eq!(Value::Int(1).access_index(Value::Int(1)), Value::Null);
    }

    #[test]
    fn test_value_eq() {
        assert_eq!(Value::Int(1), Value::Int(1));
        assert_eq!(Value::Float(1.0), Value::Float(1.0));
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_eq!(
            Value::String("Hello".to_string()),
            Value::String("Hello".to_string())
        );
        assert_eq!(
            Value::Vec(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
            Value::Vec(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );
        assert_eq!(Value::Null, Value::Null);
        assert_eq!(Value::Void, Value::Void);
    }

    #[test]
    fn test_value_partial_cmp() {
        assert_eq!(
            Value::Int(1).partial_cmp(&Value::Int(2)),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::Float(1.0).partial_cmp(&Value::Float(2.0)),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::Int(1).partial_cmp(&Value::Float(2.0)),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            Value::Float(1.0).partial_cmp(&Value::Int(2)),
            Some(std::cmp::Ordering::Less)
        );
    }

    #[test]
    fn test_value_display() {
        assert_eq!(format!("{}", Value::Int(1)), "1");
        assert_eq!(format!("{}", Value::Float(1.0)), "1");
        assert_eq!(format!("{}", Value::Bool(true)), "true");
        assert_eq!(format!("{}", Value::String("Hello".to_string())), "Hello");
        assert_eq!(
            format!(
                "{}",
                Value::Vec(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
            ),
            "[1, 2, 3]"
        );
        assert_eq!(format!("{}", Value::Null), "null");
        assert_eq!(format!("{}", Value::Void), "void");
    }

    #[test]
    fn test_value_type_name() {
        assert_eq!(Value::Int(1).type_name(), "int");
        assert_eq!(Value::Float(1.0).type_name(), "float");
        assert_eq!(Value::Bool(true).type_name(), "bool");
        assert_eq!(Value::String("Hello".to_string()).type_name(), "string");
        assert_eq!(
            Value::Vec(vec![Value::Int(1), Value::Int(2), Value::Int(3)]).type_name(),
            "int[]"
        );
        assert_eq!(Value::Null.type_name(), "null");
        assert_eq!(Value::Void.type_name(), "void");
    }

    #[test]
    fn test_value_is_type() {
        assert!(Value::Int(1).is_type("int"));
        assert!(Value::Float(1.0).is_type("float"));
        assert!(Value::Bool(true).is_type("bool"));
        assert!(Value::String("Hello".to_string()).is_type("string"));
        // We check it outside the is_type method
        // assert!(
        //     Value::Vec(vec![Value::Int(1), Value::Int(2), Value::Int(3)]).is_type("int[]")
        // );
        assert!(Value::Null.is_type("null"));
        assert!(Value::Void.is_type("void"));
    }

    #[test]
    fn test_value_is_array() {
        assert!(Value::Vec(vec![Value::Int(1), Value::Int(2), Value::Int(3)]).is_array());
        assert!(!Value::Int(1).is_array());
    }

    #[test]
    fn test_value_is_bool() {
        assert!(Value::Bool(true).is_bool());
        assert!(!Value::Int(1).is_bool());
    }

    #[test]
    fn test_value_is_float() {
        assert!(Value::Float(1.0).is_float());
        assert!(!Value::Int(1).is_float());
    }

    #[test]
    fn test_value_is_int() {
        assert!(Value::Int(1).is_int());
        assert!(!Value::Float(1.0).is_int());
    }

    #[test]
    fn test_value_is_null() {
        assert!(Value::Null.is_null());
        assert!(!Value::Int(1).is_null());
    }

    #[test]
    fn test_value_is_string() {
        assert!(Value::String("Hello".to_string()).is_string());
        assert!(!Value::Int(1).is_string());
    }
}
