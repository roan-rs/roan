use std::fmt::{Debug, Display};
use roan_ast::{Literal, LiteralType};

#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Vec(Vec<Value>),
    Null,
    Void,
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

impl std::ops::Add for Value {
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