use std::fmt::{Debug, Formatter};
use crate::ast::expr::Expr;
use crate::{GetSpan, Token};

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Expr(Box<Expr>),
    Use(Use),
    Block(Block),
    If(If),
    Return(Return),
    Fn(Fn),
    Let(Let),
    Throw(Throw),
    Try(Try),
    // TODO: loop, continue, break
}

#[derive(Clone, Debug, PartialEq)]
pub struct Throw {
    pub value: Box<Expr>,
    pub token: Token,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Try {
    pub try_token: Token,
    pub try_block: Block,
    pub error_ident: Token,
    pub catch_block: Block,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Let {
    pub ident: Token,
    pub initializer: Box<Expr>,
    pub type_annotation: Option<TypeAnnotation>,
}

impl From<Expr> for Stmt {
    fn from(expr: Expr) -> Self {
        Stmt::Expr(Box::new(expr))
    }
}

impl Stmt {
    pub fn new_try(try_token: Token, try_block: Block, error_ident: Token, catch_block: Block) -> Self {
        Stmt::Try(Try {
            try_token,
            try_block,
            error_ident,
            catch_block,
        })
    }

    pub fn new_throw(token: Token, value: Expr) -> Self {
        Stmt::Throw(Throw {
            value: Box::new(value),
            token,
        })
    }

    pub fn new_fn(
        fn_token: Token,
        name: String,
        params: Vec<FnParam>,
        body: Block,
        exported: bool,
        return_type: Option<FunctionType>,
    ) -> Self {
        Stmt::Fn(Fn {
            fn_token,
            name,
            params,
            body,
            exported,
            return_type,
        })
    }

    pub fn new_use(use_token: Token, from: Token, items: Vec<Token>) -> Self {
        Stmt::Use(Use {
            use_token,
            from,
            items,
        })
    }

    pub fn new_if(
        if_token: Token,
        condition: Box<Expr>,
        then_block: Block,
        else_ifs: Vec<ElseBlock>,
        else_block: Option<ElseBlock>,
    ) -> Self {
        Stmt::If(If {
            if_token,
            condition,
            then_block,
            else_ifs,
            else_block,
        })
    }

    pub fn new_let(
        ident: Token,
        initializer: Box<Expr>,
        type_annotation: Option<TypeAnnotation>,
    ) -> Self {
        Stmt::Let(Let {
            ident,
            initializer,
            type_annotation,
        })
    }

    pub fn new_return(return_token: Token, expr: Option<Box<Expr>>) -> Self {
        Stmt::Return(Return { return_token, expr })
    }
}

impl Stmt {
    pub fn as_function(&self) -> &Fn {
        match self {
            Stmt::Fn(f) => f,
            _ => panic!("Expected function"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FnParam {
    pub ident: Token,
    pub type_annotation: TypeAnnotation,
    pub is_rest: bool,
}

impl FnParam {
    pub fn new(ident: Token, type_annotation: TypeAnnotation, is_rest: bool) -> Self {
        Self {
            ident,
            type_annotation,
            is_rest,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeAnnotation {
    pub colon: Token,
    pub type_name: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub arrow: Token,
    pub type_name: Token,
}

impl FunctionType {
    pub fn new(arrow: Token, type_name: Token) -> Self {
        Self { arrow, type_name }
    }
}

#[derive(Clone, PartialEq)]
pub struct Fn {
    pub fn_token: Token,
    pub name: String,
    pub params: Vec<FnParam>,
    pub body: Block,
    pub exported: bool,
    pub return_type: Option<FunctionType>,
}

impl Debug for Fn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fn")
            .field("name", &self.name)
            .field("params", &self.params)
            .field("body", &self.body)
            .field("exported", &self.exported)
            .field("return_type", &self.return_type)
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct If {
    pub if_token: Token,
    pub condition: Box<Expr>,
    pub then_block: Block,
    pub else_ifs: Vec<ElseBlock>,
    pub else_block: Option<ElseBlock>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElseBlock {
    pub condition: Box<Expr>,
    pub block: Block,
    pub else_if: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Use {
    pub use_token: Token,
    pub from: Token,
    pub items: Vec<Token>,
}

#[derive(Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

impl Debug for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Block")
            .field("stmts", &self.stmts.len())
            .finish()
    }
}

#[derive(Clone, PartialEq)]
pub struct Return {
    pub return_token: Token,
    pub expr: Option<Box<Expr>>,
}

impl Debug for Return {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Return")
            .field("expr", &self.expr.clone().unwrap().span().literal)
            .finish()
    }
}