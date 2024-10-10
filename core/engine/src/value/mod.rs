use crate::{
    value::methods::{
        string::{__string_len, __string_split},
        vec::__vec_len,
    },
    vm::native_fn::NativeFunction,
};
use roan_ast::{Literal, LiteralType};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    ops,
};

pub mod methods {
    pub mod string;
    pub mod vec;
}

#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Vec(std::vec::Vec<Value>),
    Null,
    Void,
}

impl Value {
    pub fn builtin_methods(&self) -> HashMap<String, NativeFunction> {
        match self {
            Value::Vec(_) => {
                let mut map = HashMap::new();
                map.insert("len".to_string(), __vec_len());
                map
            }
            Value::String(_) => {
                let mut map = HashMap::new();
                map.insert("len".to_string(), __string_len());
                map.insert("split".to_string(), __string_split());
                map
            }
            _ => HashMap::new(),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::Vec(v) => write!(f, "{:?}", v),
            Value::Null => write!(f, "Null"),
            Value::Void => write!(f, "Void"),
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
