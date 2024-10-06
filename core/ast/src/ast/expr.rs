use std::fmt::{Display, Formatter};
use roan_error::TextSpan;
use crate::{GetSpan, Token};
use crate::statements::Stmt;

#[derive(Clone, Debug, PartialEq)]
pub struct VecExpr {
    pub exprs: Vec<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LiteralType {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Literal {
    pub token: Token,
    pub value: LiteralType,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BinOpKind {
    // Arithmetic
    Plus, // +
    Minus, // -
    Multiply, // *
    Divide, // /
    Power, // **
    Modulo, // %
    // Bitwise
    BitwiseAnd, // &
    BitwiseOr, // |
    BitwiseXor, // ^
    // Relational
    Equals, // ==
    NotEquals, // !=
    LessThan, // <
    LessThanOrEqual, // <=
    GreaterThan, // >
    GreaterThanOrEqual, // >=
    // Logical
    And, // &&
    Or, // ||
    // Equality
    EqualsEquals, // ==
    BangEquals, // !=
    // Increment/Decrement
    Increment, // ++
    Decrement, // --
    // Assignment
    MinusEquals, // -=
    PlusEquals, // +=
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: BinOpKind,
    pub right: Box<Expr>,
}

impl GetSpan for Binary {
    fn span(&self) -> TextSpan {
        let left = self.left.span();
        let right = self.right.span();

        TextSpan::combine(vec![left, right])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unary {
    pub operator: UnOperator,
    pub expr: Box<Expr>,
    pub token: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub ident: String,
    pub token: Token,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum LogicalOp {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Logical {
    pub left: Box<Expr>,
    pub operator: LogicalOp,
    pub right: Box<Expr>,
    pub token: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parenthesized {
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    pub callee: String,
    pub args: Vec<Expr>,
    pub token: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assign {
    pub ident: Token,
    pub value: Box<Expr>,
    pub token: Token,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UnOpKind {
    Minus,
    BitwiseNot,
}

impl Display for UnOpKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOpKind::Minus => write!(f, "-"),
            UnOpKind::BitwiseNot => write!(f, "~"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnOperator {
    pub kind: UnOpKind,
    pub token: Token,
}

impl UnOperator {
    pub fn new(kind: UnOpKind, token: Token) -> Self {
        UnOperator { kind, token }
    }
}

#[derive(Debug, Clone)]
pub struct BinOperator {
    pub kind: BinOpKind,
    pub token: Token,
}

impl BinOperator {
    pub fn new(kind: BinOpKind, token: Token) -> Self {
        BinOperator { kind, token }
    }

    pub fn precedence(&self) -> u8 {
        match self.kind {
            // Highest precedence
            BinOpKind::Power => 20,
            BinOpKind::Multiply => 19,
            BinOpKind::Divide => 19,
            BinOpKind::Modulo => 19,
            BinOpKind::Plus => 18,
            BinOpKind::Minus => 18,
            BinOpKind::BitwiseAnd => 17,
            BinOpKind::BitwiseXor => 16,
            BinOpKind::BitwiseOr => 15,
            // Relational operators
            BinOpKind::LessThan => 14,
            BinOpKind::LessThanOrEqual => 14,
            BinOpKind::GreaterThan => 14,
            BinOpKind::GreaterThanOrEqual => 14,
            // Equality operators
            BinOpKind::Equals => 13,
            BinOpKind::NotEquals => 13,
            BinOpKind::EqualsEquals => 13,
            BinOpKind::BangEquals => 13,
            // Logical operators
            BinOpKind::And => 12,
            BinOpKind::Or => 11,
            // Increment/Decrement
            BinOpKind::Increment => 10,
            BinOpKind::Decrement => 10,
            // Assignment operators
            BinOpKind::MinusEquals => 9,
            BinOpKind::PlusEquals => 9,
        }
    }

    pub fn associativity(&self) -> BinOpAssociativity {
        match self.kind {
            BinOpKind::Power => BinOpAssociativity::Right,
            _ => BinOpAssociativity::Left,
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum BinOpAssociativity {
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Binary(Binary),
    Unary(Unary),
    Variable(Variable),
    Logical(Logical),
    Parenthesized(Parenthesized),
    Call(CallExpr),
    Assign(Assign),
    Vec(VecExpr),
}

impl GetSpan for Expr {
    fn span(&self) -> TextSpan {
        match &self {
            Expr::Literal(l) => l.clone().token.span,
            Expr::Binary(b) => {
                let left = b.left.span();
                let right = b.right.span();

                TextSpan::combine(vec![left, right])
            }
            Expr::Unary(u) => u.clone().token.span,
            Expr::Variable(v) => v.clone().token.span,
            Expr::Logical(l) => l.clone().token.span,
            Expr::Parenthesized(p) => p.expr.span(),
            Expr::Call(c) => c.clone().token.span,
            Expr::Assign(a) => {
                let ident = a.clone().ident.span;
                let value = a.value.span();

                TextSpan::combine(vec![ident, value])
            }
            Expr::Vec(v) => {
                let mut spans = vec![];
                for expr in &v.exprs {
                    spans.push(expr.span());
                }

                TextSpan::combine(spans)
            }
        }
    }
}

impl Expr {
    pub fn into_stmt(self) -> Stmt {
        Stmt::Expr(Box::new(self))
    }

    pub fn into_variable(self) -> Variable {
        match self {
            Expr::Variable(v) => v,
            _ => panic!("Expected variable"),
        }
    }
}

impl Expr {
    pub fn new_unary(operator: UnOperator, expr: Expr, token: Token) -> Self {
        Expr::Unary(Unary {
            operator,
            expr: Box::new(expr),
            token,
        })
    }

    pub fn new_assign(ident: Token, token: Token, value: Expr) -> Self {
        Expr::Assign(Assign {
            ident,
            value: Box::new(value),
            token,
        })
    }

    pub fn new_binary(left: Expr, operator: BinOperator, right: Expr) -> Self {
        Expr::Binary(Binary {
            left: Box::new(left),
            operator: operator.kind,
            right: Box::new(right),
        })
    }

    pub fn new_integer(token: Token, value: i64) -> Self {
        Expr::Literal(Literal {
            token,
            value: LiteralType::Int(value),
        })
    }

    pub fn new_float(token: Token, value: f64) -> Self {
        Expr::Literal(Literal {
            token,
            value: LiteralType::Float(value),
        })
    }

    pub fn new_bool(token: Token, value: bool) -> Self {
        Expr::Literal(Literal {
            token,
            value: LiteralType::Bool(value),
        })
    }

    pub fn new_variable(ident: Token, name: String) -> Self {
        Expr::Variable(Variable {
            ident: name,
            token: ident,
        })
    }

    pub fn new_call(callee: String, args: Vec<Expr>, token: Token) -> Self {
        Expr::Call(CallExpr {
            callee,
            args,
            token,
        })
    }

    pub fn new_string(token: Token, value: String) -> Self {
        Expr::Literal(Literal {
            token,
            value: LiteralType::String(value),
        })
    }

    pub fn new_parenthesized(expr: Expr) -> Self {
        Expr::Parenthesized(Parenthesized {
            expr: Box::new(expr),
        })
    }

    pub fn new_vec(exprs: Vec<Expr>) -> Self {
        Expr::Vec(VecExpr { exprs })
    }
}
