use std::fmt::{Display, Formatter};
use roan_error::TextSpan;
use crate::{GetSpan, Token};
use crate::statements::Stmt;

/// Represents a collection of expressions as a vector.
/// Used to handle lists of expressions, such as arrays or argument lists.
#[derive(Clone, Debug, PartialEq)]
pub struct VecExpr {
    /// The vector containing the expressions.
    pub exprs: Vec<Expr>,
}

/// Enum that defines the possible literal types in the language.
/// Literals are constant values such as numbers, strings, and booleans.
#[derive(Clone, Debug, PartialEq)]
pub enum LiteralType {
    /// An integer literal (e.g., `42`).
    Int(i64),
    /// A floating-point literal (e.g., `3.14`).
    Float(f64),
    /// A string literal (e.g., `"hello"`).
    String(String),
    /// A boolean literal (`true` or `false`).
    Bool(bool),
    /// A `null` literal representing the absence of a value.
    Null,
}

/// Represents a literal expression in the AST.
/// It consists of a token and a specific literal value (e.g., integer, string).
#[derive(Clone, Debug, PartialEq)]
pub struct Literal {
    /// The token representing the literal in the source code.
    pub token: Token,
    /// The value of the literal.
    pub value: LiteralType,
}

/// Enum representing the various binary operators in the language.
/// Binary operators are used in binary expressions (e.g., `a + b`).
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BinOpKind {
    // Arithmetic operators
    /// Addition operator (`+`).
    Plus,
    /// Subtraction operator (`-`).
    Minus,
    /// Multiplication operator (`*`).
    Multiply,
    /// Division operator (`/`).
    Divide,
    /// Exponentiation operator (`**`).
    Power,
    /// Modulo operator (`%`).
    Modulo,
    // Bitwise operators
    /// Bitwise AND operator (`&`).
    BitwiseAnd,
    /// Bitwise OR operator (`|`).
    BitwiseOr,
    /// Bitwise XOR operator (`^`).
    BitwiseXor,
    // Relational operators
    /// Equality operator (`==`).
    Equals,
    /// Inequality operator (`!=`).
    NotEquals,
    /// Less-than operator (`<`).
    LessThan,
    /// Less-than-or-equal operator (`<=`).
    LessThanOrEqual,
    /// Greater-than operator (`>`).
    GreaterThan,
    /// Greater-than-or-equal operator (`>=`).
    GreaterThanOrEqual,
    // Logical operators
    /// Logical AND operator (`&&`).
    And,
    /// Logical OR operator (`||`).
    Or,
    // Equality operators (duplicated? Consider removing duplicates)
    /// Equality operator (`==`).
    EqualsEquals,
    /// Inequality operator (`!=`).
    BangEquals,
    // Increment/Decrement operators
    /// Increment operator (`++`).
    Increment,
    /// Decrement operator (`--`).
    Decrement,
    // Assignment operators
    /// Subtraction assignment operator (`-=`).
    MinusEquals,
    /// Addition assignment operator (`+=`).
    PlusEquals,
}

/// Represents a binary expression in the AST.
/// A binary expression consists of two operands and an operator (e.g., `a + b`).
#[derive(Debug, Clone, PartialEq)]
pub struct Binary {
    /// The left operand of the binary expression.
    pub left: Box<Expr>,
    /// The operator used in the binary expression.
    pub operator: BinOpKind,
    /// The right operand of the binary expression.
    pub right: Box<Expr>,
}

impl GetSpan for Binary {
    /// Returns the combined source span of the left and right operands.
    fn span(&self) -> TextSpan {
        let left = self.left.span();
        let right = self.right.span();

        TextSpan::combine(vec![left, right])
    }
}

/// Represents a unary expression in the AST.
/// Unary expressions operate on a single operand (e.g., `-a`).
#[derive(Debug, Clone, PartialEq)]
pub struct Unary {
    /// The operator used in the unary expression.
    pub operator: UnOperator,
    /// The operand of the unary expression.
    pub expr: Box<Expr>,
    /// The token representing the unary operation in the source code.
    pub token: Token,
}

/// Represents a variable in the AST.
/// A variable refers to a named entity in the program.
#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    /// The name of the variable.
    pub ident: String,
    /// The token representing the variable in the source code.
    pub token: Token,
}

/// Enum representing logical operators such as `&&` and `||`.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum LogicalOp {
    /// Logical AND operator (`&&`).
    And,
    /// Logical OR operator (`||`).
    Or,
}

/// Represents a logical expression in the AST.
/// Logical expressions include operators like `&&` and `||` that work on boolean values.
#[derive(Debug, Clone, PartialEq)]
pub struct Logical {
    /// The left operand of the logical expression.
    pub left: Box<Expr>,
    /// The logical operator (`&&` or `||`).
    pub operator: LogicalOp,
    /// The right operand of the logical expression.
    pub right: Box<Expr>,
    /// The token representing the logical operation in the source code.
    pub token: Token,
}

/// Represents a parenthesized expression in the AST.
/// Parenthesized expressions are used to override operator precedence.
#[derive(Debug, Clone, PartialEq)]
pub struct Parenthesized {
    /// The expression contained within the parentheses.
    pub expr: Box<Expr>,
}

/// Represents a function call in the AST.
/// It consists of a function name (callee) and a list of arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    /// The name of the function being called.
    pub callee: String,
    /// The list of arguments passed to the function.
    pub args: Vec<Expr>,
    /// The token representing the function call in the source code.
    pub token: Token,
}

/// Represents an assignment expression in the AST.
/// An assignment binds a value to a variable.
#[derive(Debug, Clone, PartialEq)]
pub struct Assign {
    /// The token representing the identifier (variable name).
    pub ident: Token,
    /// The value being assigned to the identifier.
    pub value: Box<Expr>,
    /// The token representing the assignment operation.
    pub token: Token,
}

/// Enum representing unary operator kinds (e.g., `-`, `~`).
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UnOpKind {
    /// Minus operator (`-`).
    Minus,
    /// Bitwise NOT operator (`~`).
    BitwiseNot,
}

impl Display for UnOpKind {
    /// Formats the unary operator as a string.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOpKind::Minus => write!(f, "-"),
            UnOpKind::BitwiseNot => write!(f, "~"),
        }
    }
}

/// Represents a unary operator in the AST.
#[derive(Debug, Clone, PartialEq)]
pub struct UnOperator {
    /// The specific kind of unary operator.
    pub kind: UnOpKind,
    /// The token representing the unary operator in the source code.
    pub token: Token,
}

impl UnOperator {
    /// Creates a new unary operator.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of unary operator.
    /// * `token` - The token representing the operator in the source code.
    ///
    /// # Returns
    ///
    /// A new `UnOperator` instance.
    pub fn new(kind: UnOpKind, token: Token) -> Self {
        UnOperator { kind, token }
    }
}

/// Represents a binary operator in the AST.
#[derive(Debug, Clone)]
pub struct BinOperator {
    /// The specific kind of binary operator.
    pub kind: BinOpKind,
    /// The token representing the binary operator in the source code.
    pub token: Token,
}

impl BinOperator {
    /// Creates a new binary operator.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of binary operator.
    /// * `token` - The token representing the operator in the source code.
    ///
    /// # Returns
    ///
    /// A new `BinOperator` instance.
    pub fn new(kind: BinOpKind, token: Token) -> Self {
        BinOperator { kind, token }
    }

    /// Returns the precedence of the operator.
    ///
    /// Higher numbers indicate higher precedence.
    ///
    /// # Returns
    ///
    /// An unsigned 8-bit integer representing the operator's precedence.
    pub fn precedence(&self) -> u8 {
        match self.kind {
            // Highest precedence
            BinOpKind::Power => 20,
            BinOpKind::Multiply | BinOpKind::Divide | BinOpKind::Modulo => 19,
            BinOpKind::Plus | BinOpKind::Minus => 18,
            BinOpKind::BitwiseAnd => 17,
            BinOpKind::BitwiseXor => 16,
            BinOpKind::BitwiseOr => 15,
            // Relational
            BinOpKind::LessThan | BinOpKind::LessThanOrEqual |
            BinOpKind::GreaterThan | BinOpKind::GreaterThanOrEqual => 14,
            // Equality
            BinOpKind::Equals | BinOpKind::NotEquals |
            BinOpKind::EqualsEquals | BinOpKind::BangEquals => 13,
            // Logical
            BinOpKind::And => 12,
            BinOpKind::Or => 11,
            // Increment/Decrement
            BinOpKind::Increment | BinOpKind::Decrement => 10,
            // Assignment
            BinOpKind::MinusEquals | BinOpKind::PlusEquals => 9,
        }
    }

    /// Returns the associativity of the operator.
    ///
    /// Operators can be either left-associative or right-associative.
    ///
    /// # Returns
    ///
    /// A `BinOpAssociativity` enum indicating the associativity.
    pub fn associativity(&self) -> BinOpAssociativity {
        match self.kind {
            BinOpKind::Power => BinOpAssociativity::Right,
            _ => BinOpAssociativity::Left,
        }
    }
}

/// Enum representing the associativity of a binary operator.
#[derive(Debug, Clone, PartialEq)]
pub enum BinOpAssociativity {
    /// Left-associative operators group from the left.
    Left,
    /// Right-associative operators group from the right.
    Right,
}

/// Enum representing an expression in the AST.
/// Expressions include literals, binary operations, unary operations, and more.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// A literal value (e.g., number, string).
    Literal(Literal),
    /// A binary operation (e.g., `a + b`).
    Binary(Binary),
    /// A unary operation (e.g., `-a`).
    Unary(Unary),
    /// A variable reference.
    Variable(Variable),
    /// A logical operation (e.g., `a && b`).
    Logical(Logical),
    /// A parenthesized expression to override precedence.
    Parenthesized(Parenthesized),
    /// A function call expression.
    Call(CallExpr),
    /// An assignment expression (e.g., `a = b`).
    Assign(Assign),
    /// A vector (list) of expressions.
    Vec(VecExpr),
}

impl GetSpan for Expr {
    /// Returns the `TextSpan` associated with the expression in the source code.
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
                let spans: Vec<TextSpan> = v.exprs.iter().map(|e| e.span()).collect();
                TextSpan::combine(spans)
            }
        }
    }
}

impl Expr {
    /// Converts the expression into a statement.
    ///
    /// # Returns
    ///
    /// A `Stmt::Expr` variant containing the expression.
    pub fn into_stmt(self) -> Stmt {
        Stmt::Expr(Box::new(self))
    }

    /// Converts the expression into a variable.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a `Variable` variant.
    ///
    /// # Returns
    ///
    /// The `Variable` struct contained within the expression.
    pub fn into_variable(self) -> Variable {
        match self {
            Expr::Variable(v) => v,
            _ => panic!("Expected variable"),
        }
    }

    /// Creates a new unary expression.
    ///
    /// # Arguments
    ///
    /// * `operator` - The unary operator.
    /// * `expr` - The operand expression.
    /// * `token` - The token representing the unary operation.
    ///
    /// # Returns
    ///
    /// A new `Expr::Unary` variant.
    pub fn new_unary(operator: UnOperator, expr: Expr, token: Token) -> Self {
        Expr::Unary(Unary {
            operator,
            expr: Box::new(expr),
            token,
        })
    }

    /// Creates a new assignment expression.
    ///
    /// # Arguments
    ///
    /// * `ident` - The token representing the variable identifier.
    /// * `token` - The token representing the assignment operation.
    /// * `value` - The expression to assign.
    ///
    /// # Returns
    ///
    /// A new `Expr::Assign` variant.
    pub fn new_assign(ident: Token, token: Token, value: Expr) -> Self {
        Expr::Assign(Assign {
            ident,
            value: Box::new(value),
            token,
        })
    }

    /// Creates a new binary expression.
    ///
    /// # Arguments
    ///
    /// * `left` - The left operand expression.
    /// * `operator` - The binary operator.
    /// * `right` - The right operand expression.
    ///
    /// # Returns
    ///
    /// A new `Expr::Binary` variant.
    pub fn new_binary(left: Expr, operator: BinOperator, right: Expr) -> Self {
        Expr::Binary(Binary {
            left: Box::new(left),
            operator: operator.kind,
            right: Box::new(right),
        })
    }

    /// Creates a new integer literal expression.
    ///
    /// # Arguments
    ///
    /// * `token` - The token representing the integer literal.
    /// * `value` - The integer value.
    ///
    /// # Returns
    ///
    /// A new `Expr::Literal` variant with `LiteralType::Int`.
    pub fn new_integer(token: Token, value: i64) -> Self {
        Expr::Literal(Literal {
            token,
            value: LiteralType::Int(value),
        })
    }

    /// Creates a new floating-point literal expression.
    ///
    /// # Arguments
    ///
    /// * `token` - The token representing the float literal.
    /// * `value` - The floating-point value.
    ///
    /// # Returns
    ///
    /// A new `Expr::Literal` variant with `LiteralType::Float`.
    pub fn new_float(token: Token, value: f64) -> Self {
        Expr::Literal(Literal {
            token,
            value: LiteralType::Float(value),
        })
    }

    /// Creates a new boolean literal expression.
    ///
    /// # Arguments
    ///
    /// * `token` - The token representing the boolean literal.
    /// * `value` - The boolean value.
    ///
    /// # Returns
    ///
    /// A new `Expr::Literal` variant with `LiteralType::Bool`.
    pub fn new_bool(token: Token, value: bool) -> Self {
        Expr::Literal(Literal {
            token,
            value: LiteralType::Bool(value),
        })
    }

    /// Creates a new variable expression.
    ///
    /// # Arguments
    ///
    /// * `ident` - The token representing the variable identifier.
    /// * `name` - The name of the variable.
    ///
    /// # Returns
    ///
    /// A new `Expr::Variable` variant.
    pub fn new_variable(ident: Token, name: String) -> Self {
        Expr::Variable(Variable {
            ident: name,
            token: ident,
        })
    }

    /// Creates a new function call expression.
    ///
    /// # Arguments
    ///
    /// * `callee` - The name of the function being called.
    /// * `args` - The list of argument expressions.
    /// * `token` - The token representing the function call.
    ///
    /// # Returns
    ///
    /// A new `Expr::Call` variant.
    pub fn new_call(callee: String, args: Vec<Expr>, token: Token) -> Self {
        Expr::Call(CallExpr {
            callee,
            args,
            token,
        })
    }

    /// Creates a new string literal expression.
    ///
    /// # Arguments
    ///
    /// * `token` - The token representing the string literal.
    /// * `value` - The string value.
    ///
    /// # Returns
    ///
    /// A new `Expr::Literal` variant with `LiteralType::String`.
    pub fn new_string(token: Token, value: String) -> Self {
        Expr::Literal(Literal {
            token,
            value: LiteralType::String(value),
        })
    }

    /// Creates a new parenthesized expression.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression to be parenthesized.
    ///
    /// # Returns
    ///
    /// A new `Expr::Parenthesized` variant.
    pub fn new_parenthesized(expr: Expr) -> Self {
        Expr::Parenthesized(Parenthesized {
            expr: Box::new(expr),
        })
    }

    /// Creates a new vector expression.
    ///
    /// # Arguments
    ///
    /// * `exprs` - The list of expressions in the vector.
    ///
    /// # Returns
    ///
    /// A new `Expr::Vec` variant.
    pub fn new_vec(exprs: Vec<Expr>) -> Self {
        Expr::Vec(VecExpr { exprs })
    }
}
