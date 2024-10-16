use crate::{
    entries,
    value::methods::{
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
use roan_ast::{Literal, LiteralType, Struct};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    ops,
};

pub mod methods {
    pub mod string;
    pub mod vec;
}

#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Vec(Vec<Value>),
    Struct(Struct, HashMap<String, Value>),
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

                    // TODO: Implement static methods
                    // "to_int" => __string_to_int(),
                    // "to_float" => __string_to_float(),
                    // "to_bool" => __string_to_bool(),
                    // "from_int" => __string_from_int(),
                    // "from_float" => __string_from_float(),
                    // "from_bool" => __string_from_bool(),
                    // "from_code" => __string_from_code()
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
        }
    }
}

impl ops::Add for Value {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
            (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 + b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a + b as f64),
            (Value::String(a), Value::String(b)) => Value::String(a + &b),
            _ => panic!("Cannot add values of different types"),
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
            // TODO: improve formatting of structs
            Value::Struct(..) => write!(f, "struct"),
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
            _ => Value::Null,
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
}
