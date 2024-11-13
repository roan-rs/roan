#![feature(let_chains)]
#[allow(unused_mut)]
extern crate core;

pub mod context;
pub mod interpreter;
mod macros;
pub mod module;
pub mod natives;
pub mod path;
pub mod value;
pub mod vm;

pub use roan_ast::*;
pub use roan_error::{diagnostic::*, error::RoanError::*, span::*};
