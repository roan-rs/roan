use roan_error::TextSpan;

pub mod statements;
pub mod expr;

#[derive(Clone, Debug, PartialEq)]
pub struct Ast {
    pub stmts: Vec<Stmt>,
}

impl Ast {
    pub fn new() -> Self {
        Self { stmts: vec![] }
    }
}

pub trait GetSpan {
    fn span(&self) -> TextSpan;
}

pub use statements::*;
pub use expr::*;