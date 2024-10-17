use crate::{statements::Stmt, GetSpan, Token, TokenKind};
use roan_error::TextSpan;
use std::fmt::{Display, Formatter};

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

impl Literal {
    pub fn new(token: Token, value: LiteralType) -> Self {
        Literal { token, value }
    }
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
    /// Bitwise shift left operator (`<<`).
    ShiftLeft,
    /// Bitwise shift right operator (`>>`).
    ShiftRight,
    // Relational operators
    /// Equality operator (`==`).
    Equals,
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

        TextSpan::combine(vec![left, right]).unwrap()
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

impl GetSpan for Unary {
    /// Returns the source span of the unary expression.
    fn span(&self) -> TextSpan {
        TextSpan::combine(vec![self.operator.token.span.clone(), self.expr.span()]).unwrap()
    }
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

impl GetSpan for CallExpr {
    /// Returns the source span of the function call expression.
    fn span(&self) -> TextSpan {
        // TODO: get the span of the closing parenthesis
        let callee_span = self.token.span.clone();
        let args_span: Vec<TextSpan> = self.args.iter().map(|arg| arg.span()).collect();
        let args_span = TextSpan::combine(args_span);

        let mut spans = vec![callee_span];

        if args_span.is_some() {
            spans.push(args_span.unwrap());
        }

        TextSpan::combine(spans).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOperator {
    /// Assignment operator (`=`).
    Assign,
    /// Addition assignment operator (`+=`).
    PlusEquals,
    /// Subtraction assignment operator (`-=`).
    MinusEquals,
    /// Multiplication assignment operator (`*=`).
    MultiplyEquals,
    /// Division assignment operator (`/=`).
    DivideEquals,
}

impl AssignOperator {
    pub fn from_token_kind(kind: TokenKind) -> Self {
        match kind {
            TokenKind::Equals => AssignOperator::Assign,
            TokenKind::PlusEquals => AssignOperator::PlusEquals,
            TokenKind::MinusEquals => AssignOperator::MinusEquals,
            TokenKind::MultiplyEquals => AssignOperator::MultiplyEquals,
            TokenKind::DivideEquals => AssignOperator::DivideEquals,
            _ => todo!("Proper error"),
        }
    }
}

/// Represents an assignment expression in the AST.
/// An assignment binds a value to a variable.
#[derive(Debug, Clone, PartialEq)]
pub struct Assign {
    /// The variable being assigned to.
    pub left: Box<Expr>,
    /// The assignment operator.
    pub op: AssignOperator,
    /// The value being assigned.
    pub right: Box<Expr>,
}

/// Enum representing unary operator kinds (e.g., `-`, `~`).
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UnOpKind {
    /// Minus operator (`-`).
    Minus,
    /// Bitwise NOT operator (`~`).
    BitwiseNot,
    /// Logical NOT operator (`!`).
    LogicalNot,
}

impl Display for UnOpKind {
    /// Formats the unary operator as a string.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOpKind::Minus => write!(f, "-"),
            UnOpKind::BitwiseNot => write!(f, "~"),
            UnOpKind::LogicalNot => write!(f, "!"),
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
            BinOpKind::ShiftLeft | BinOpKind::ShiftRight => 17,
            BinOpKind::BitwiseAnd => 16,
            BinOpKind::BitwiseXor => 15,
            BinOpKind::BitwiseOr => 14,
            // Relational operators
            BinOpKind::LessThan
            | BinOpKind::LessThanOrEqual
            | BinOpKind::GreaterThan
            | BinOpKind::GreaterThanOrEqual => 13,
            // Equality operators
            BinOpKind::Equals | BinOpKind::EqualsEquals | BinOpKind::BangEquals => 12,
            // Logical operators
            BinOpKind::And => 11,
            BinOpKind::Or => 10,
            // Increment/Decrement operators
            BinOpKind::Increment | BinOpKind::Decrement => 9,
            // Assignment operators
            BinOpKind::MinusEquals | BinOpKind::PlusEquals => 8,
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

/// Spread operator for variadic arguments.
///
/// The spread operator is used to pass an array as separate arguments to a function.
#[derive(Debug, Clone, PartialEq)]
pub struct Spread {
    pub token: Token,
    pub expr: Box<Expr>,
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
    /// A parenthesized expression to override precedence.
    Parenthesized(Parenthesized),
    /// A function call expression.
    Call(CallExpr),
    /// An assignment expression (e.g., `a = b`).
    Assign(Assign),
    /// A vector (list) of expressions.
    Vec(VecExpr),
    /// An access expression (e.g., `struct.name`, `arr[0]`, `Person::new`).
    Access(AccessExpr),
    /// A spread operator for variadic arguments. (e.g., `...args`)
    Spread(Spread),
    /// Null literal.
    Null(Token),
    /// Struct constructor. (e.g., `MyStruct { field: value }`)
    StructConstructor(StructConstructor),
    /// Then-else expression. (e.g., `if condition then value else other`)
    ThenElse(ThenElse),
}

/// Represents a then-else expression in the AST.
///
/// A then-else expression is used to conditionally evaluate one of two expressions.
///
/// # Examples
/// ```roan
/// let value = if condition then 42 else 0
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ThenElse {
    /// The condition expression.
    pub condition: Box<Expr>,
    /// The expression to evaluate if the condition is true.
    pub then_expr: Box<Expr>,
    /// The expression to evaluate if the condition is false.
    pub else_expr: Box<Expr>,
    /// The token representing the `then` keyword.
    pub then_token: Token,
    /// The token representing the `else` keyword.
    pub else_token: Token,
}

impl GetSpan for ThenElse {
    /// Returns the combined source span of the condition, then expression, and else expression.
    fn span(&self) -> TextSpan {
        let condition_span = self.condition.span();
        let then_span = self.then_expr.span();
        let else_span = self.else_expr.span();

        TextSpan::combine(vec![condition_span, then_span, else_span]).unwrap()
    }
}

/// Represents a struct constructor expression in the AST.
///
/// A struct constructor creates a new instance of a struct with the specified field values.
#[derive(Debug, Clone, PartialEq)]
pub struct StructConstructor {
    /// The name of the struct being constructed.
    pub name: String,
    /// The field values for the struct.
    pub fields: Vec<(String, Expr)>,
    /// The token representing the struct constructor in the source code.
    pub token: Token,
}

/// Enum representing the kind of access in an access expression.
#[derive(Debug, Clone, PartialEq)]
pub enum AccessKind {
    /// Field access (e.g., `.name`).
    Field(Box<Expr>),
    /// Index access (e.g., `[0]`).
    Index(Box<Expr>),
    /// Static method access (e.g., `Person::new`).
    StaticMethod(Box<Expr>),
}

/// Represents an access expression in the AST.
/// It includes accessing a field or indexing into a collection.
#[derive(Debug, Clone, PartialEq)]
pub struct AccessExpr {
    /// The base expression being accessed.
    pub base: Box<Expr>,
    /// The kind of access (field or index).
    pub access: AccessKind,
    /// The token representing the access operation (e.g., `.`, `[`, `]`).
    pub token: Token,
}

impl GetSpan for AccessExpr {
    /// Returns the combined source span of the base expression and the access operation.
    fn span(&self) -> TextSpan {
        let base_span = self.base.span();
        let access_span = match &self.access {
            AccessKind::Field(_) => self.token.span.clone(), // Span includes the '.' and the field name
            AccessKind::Index(index_expr) => {
                TextSpan::combine(vec![self.token.span.clone(), index_expr.span()]).unwrap()
            } // Span includes '[' , index, and ']'
            AccessKind::StaticMethod(method) => {
                TextSpan::combine(vec![self.token.span.clone(), method.span()]).unwrap()
            }
        };
        TextSpan::combine(vec![base_span, access_span]).unwrap()
    }
}

impl GetSpan for Expr {
    /// Returns the `TextSpan` associated with the expression in the source code.
    fn span(&self) -> TextSpan {
        match &self {
            Expr::Literal(l) => l.clone().token.span,
            Expr::Binary(b) => {
                let left = b.left.span();
                let right = b.right.span();
                TextSpan::combine(vec![left, right]).unwrap()
            }
            Expr::Unary(u) => u.clone().token.span,
            Expr::Variable(v) => v.clone().token.span,
            Expr::Parenthesized(p) => p.expr.span(),
            Expr::Call(c) => c.clone().token.span,
            Expr::Assign(a) => {
                let left = a.left.span();
                let right = a.right.span();
                TextSpan::combine(vec![left, right]).unwrap()
            }
            Expr::Vec(v) => {
                let spans: Vec<TextSpan> = v.exprs.iter().map(|e| e.span()).collect();
                TextSpan::combine(spans).unwrap()
            }
            Expr::Access(a) => a.span(),
            Expr::Spread(s) => {
                TextSpan::combine(vec![s.token.span.clone(), s.expr.span()]).unwrap()
            }
            Expr::Null(t) => t.span.clone(),
            Expr::StructConstructor(s) => s.token.span.clone(),
            Expr::ThenElse(t) => t.span(),
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

    /// Creates a new null literal expression.
    pub fn new_null(token: Token) -> Self {
        Expr::Null(token)
    }

    /// Creates a new field access expression.
    ///
    /// # Arguments
    ///
    /// * `base` - The base expression being accessed.
    /// * `field` - The name of the field to access.
    /// * `token` - The token representing the '.' operator.
    ///
    /// # Returns
    ///
    /// A new `Expr::Access` variant with `AccessKind::Field`.
    pub fn new_field_access(base: Expr, field: Expr, token: Token) -> Self {
        Expr::Access(AccessExpr {
            base: Box::new(base),
            access: AccessKind::Field(Box::new(field)),
            token,
        })
    }

    /// Create a new spread expression.
    ///
    /// # Arguments
    /// * `token` - The token representing the spread operator.
    /// * `expr` - The expression to spread.
    ///
    /// # Returns
    /// A new `Expr::Spread` variant.
    pub fn new_spread(token: Token, expr: Expr) -> Self {
        Expr::Spread(Spread {
            token,
            expr: Box::new(expr),
        })
    }

    /// Creates a new index access expression.
    ///
    /// # Arguments
    ///
    /// * `base` - The base expression being accessed.
    /// * `index` - The index expression.
    /// * `token` - The token representing the '[' and ']' operators.
    ///
    /// # Returns
    ///
    /// A new `Expr::Access` variant with `AccessKind::Index`.
    pub fn new_index_access(base: Expr, index: Expr, token: Token) -> Self {
        // **Added**
        Expr::Access(AccessExpr {
            base: Box::new(base),
            access: AccessKind::Index(Box::new(index)),
            token,
        })
    }

    /// Creates a new then-else expression.
    ///
    /// # Arguments
    /// * `condition` - The condition expression.
    /// * `then_expr` - The expression to evaluate if the condition is true.
    /// * `else_expr` - The expression to evaluate if the condition is false.
    /// * `then_token` - The token representing the `then` keyword.
    /// * `else_token` - The token representing the `else` keyword.
    ///
    /// # Returns
    ///
    /// A new `Expr::ThenElse` variant.
    pub fn new_then_else(
        condition: Expr,
        then_expr: Expr,
        else_expr: Expr,
        then_token: Token,
        else_token: Token,
    ) -> Self {
        Expr::ThenElse(ThenElse {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_expr: Box::new(else_expr),
            then_token,
            else_token,
        })
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
    pub fn new_assign(left: Expr, op: AssignOperator, right: Expr) -> Self {
        Expr::Assign(Assign {
            left: Box::new(left),
            op,
            right: Box::new(right),
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

    /// Creates a new struct constructor expression.
    ///
    /// # Arguments
    /// * `name` - The name of the struct being constructed.
    /// * `fields` - The field values for the struct.
    /// * `token` - The token representing the struct constructor.
    ///
    /// # Returns
    ///
    /// A new `Expr::StructConstructor` variant.
    pub fn new_struct_constructor(name: String, fields: Vec<(String, Expr)>, token: Token) -> Self {
        Expr::StructConstructor(StructConstructor {
            name,
            fields,
            token,
        })
    }

    /// Creates a new static method access expression.
    ///
    /// # Arguments
    /// * `base` - The base expression being accessed.
    /// * `method` - The method expression.
    /// * `token` - The token representing the '::' operator.
    ///
    /// # Returns
    ///
    /// A new `Expr::Access` variant with `AccessKind::StaticMethod`.
    pub fn new_static_method_access(base: Expr, method: Expr, token: Token) -> Self {
        Expr::Access(AccessExpr {
            base: Box::new(base),
            access: AccessKind::StaticMethod(Box::new(method)),
            token,
        })
    }
}
