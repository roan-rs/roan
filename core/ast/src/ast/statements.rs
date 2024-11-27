use crate::{ast::expr::Expr, GetSpan, Token};
use indexmap::IndexMap;
use roan_error::TextSpan;
use std::fmt::{Debug, Display, Formatter};

/// Represents a statement in the AST.
///
/// A statement can be an expression, a declaration, a control flow construct, etc.
#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    /// An expression statement.
    Expr(Box<Expr>),
    /// A `use` statement for importing modules or items.
    Use(Use),
    /// A block of statements enclosed in braces.
    Block(Block),
    /// An `if` statement with optional `else if` and `else` blocks.
    If(If),
    /// A `return` statement to exit a function.
    Return(Return),
    /// A function declaration.
    Fn(Fn),
    /// A variable declaration.
    Let(Let),
    /// A `throw` statement for exception handling.
    Throw(Throw),
    /// A `try` statement for handling errors.
    Try(Try),
    /// A `break` statement to exit a loop.
    Break(Token),
    /// A `continue` statement to skip the current iteration of a loop.
    Continue(Token),
    /// A `loop` statement to create an infinite loop.
    Loop(Loop),
    /// A `while` statement to create a loop with a condition.
    While(While),
    /// A struct definition.
    Struct(Struct),
    /// A trait definition.
    TraitDef(TraitDef),
    /// A struct implementation.
    StructImpl(StructImpl),
    /// A trait implementation.
    TraitImpl(TraitImpl),
    /// A const statement
    Const(Const),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Const {
    pub expr: Box<Expr>,
    pub ident: Token,
    pub public: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Struct {
    pub struct_token: Token,
    pub name: Token,
    pub fields: IndexMap<String, StructField>,
    pub public: bool,
    pub impls: Vec<StructImpl>,
    pub trait_impls: Vec<TraitImpl>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StructField {
    pub ident: Token,
    pub type_annotation: TypeAnnotation,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TraitDef {
    pub trait_token: Token,
    pub name: Token,
    pub methods: Vec<Fn>,
    pub public: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StructImpl {
    pub impl_token: Token,
    pub struct_name: Token,
    pub methods: Vec<Fn>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TraitImpl {
    pub impl_token: Token,
    pub trait_name: Token,
    pub for_token: Token,
    pub struct_name: Token,
    pub methods: Vec<Fn>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Loop {
    pub loop_token: Token,
    pub block: Block,
}

#[derive(Clone, Debug, PartialEq)]
pub struct While {
    pub while_token: Token,
    pub condition: Box<Expr>,
    pub block: Block,
}

/// Represents a `throw` statement in the AST.
///
/// The `throw` statement is used to raise an exception with a specified value.
#[derive(Clone, Debug, PartialEq)]
pub struct Throw {
    /// The expression representing the value to be thrown.
    pub value: Box<Expr>,
    /// The token corresponding to the `throw` keyword in the source code.
    pub token: Token,
}

/// Represents a `try` statement in the AST.
///
/// The `try` statement is used for error handling, allowing execution of a block of code
/// and catching any errors that occur.
#[derive(Clone, Debug, PartialEq)]
pub struct Try {
    /// The token corresponding to the `try` keyword in the source code.
    pub try_token: Token,
    /// The block of code to execute within the `try` statement.
    pub try_block: Block,
    /// The identifier token for the caught error.
    pub error_ident: Token,
    /// The block of code to execute if an error is caught.
    pub catch_block: Block,
}

/// Represents a variable declaration (`let` statement) in the AST.
///
/// A `let` statement declares a new variable with an optional type annotation and initializer.
#[derive(Clone, Debug, PartialEq)]
pub struct Let {
    /// The token representing the identifier (variable name).
    pub ident: Token,
    /// The expression used to initialize the variable.
    pub initializer: Box<Expr>,
    /// An optional type annotation specifying the type of the variable. Can be inferred.
    pub type_annotation: Option<TypeAnnotation>,
}

impl GetSpan for Let {
    fn span(&self) -> TextSpan {
        let mut spans = vec![self.ident.span.clone()];

        if let Some(type_annotation) = &self.type_annotation {
            spans.push(type_annotation.span());
        }

        spans.push(self.initializer.span());

        TextSpan::combine(spans).unwrap()
    }
}

impl From<Expr> for Stmt {
    /// Converts an `Expr` into a `Stmt::Expr`.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression to convert into a statement.
    ///
    /// # Returns
    ///
    /// A `Stmt::Expr` variant containing the provided expression.
    fn from(expr: Expr) -> Self {
        Stmt::Expr(Box::new(expr))
    }
}

impl Stmt {
    pub fn into_function(self) -> Fn {
        match self {
            Stmt::Fn(f) => f,
            _ => panic!("Expected function"),
        }
    }

    /// Creates a new `Loop` statement.
    ///
    /// # Arguments
    /// * `loop_token` - The token representing the `loop` keyword.
    /// * `block` - The block of code to execute within the loop.
    ///
    /// # Returns
    /// A `Stmt::Loop` variant containing the provided components.
    pub fn new_loop(loop_token: Token, block: Block) -> Self {
        Stmt::Loop(Loop { loop_token, block })
    }

    /// Creates a new `While` statement.
    ///
    /// # Arguments
    /// * `while_token` - The token representing the `while` keyword.
    /// * `condition` - The expression to evaluate as the loop condition.
    /// * `block` - The block of code to execute within the loop.
    ///
    /// # Returns
    /// A `Stmt::While` variant containing the provided components.
    pub fn new_while(while_token: Token, condition: Expr, block: Block) -> Self {
        Stmt::While(While {
            while_token,
            condition: Box::new(condition),
            block,
        })
    }

    /// Creates a new `Break` statement.
    ///
    /// # Arguments
    /// * `break_token` - The token representing the `break` keyword.
    ///
    /// # Returns
    /// A `Stmt::Break` variant containing the provided token.
    pub fn new_break(break_token: Token) -> Self {
        Stmt::Break(break_token)
    }

    /// Creates a new `Continue` statement.
    ///
    /// # Arguments
    /// * `continue_token` - The token representing the `continue` keyword.
    ///
    /// # Returns
    /// A `Stmt::Continue` variant containing the provided token.
    pub fn new_continue(continue_token: Token) -> Self {
        Stmt::Continue(continue_token)
    }

    /// Creates a new `Try` statement.
    ///
    /// # Arguments
    ///
    /// * `try_token` - The token for the `try` keyword.
    /// * `try_block` - The block of code to execute within the `try`.
    /// * `error_ident` - The identifier token for the caught error.
    /// * `catch_block` - The block of code to execute if an error is caught.
    ///
    /// # Returns
    ///
    /// A `Stmt::Try` variant containing the provided components.
    pub fn new_try(
        try_token: Token,
        try_block: Block,
        error_ident: Token,
        catch_block: Block,
    ) -> Self {
        Stmt::Try(Try {
            try_token,
            try_block,
            error_ident,
            catch_block,
        })
    }

    /// Creates a new `Throw` statement.
    ///
    /// # Arguments
    ///
    /// * `token` - The token representing the `throw` keyword.
    /// * `value` - The expression to be thrown.
    ///
    /// # Returns
    ///
    /// A `Stmt::Throw` variant containing the provided value and token.
    pub fn new_throw(token: Token, value: Expr) -> Self {
        Stmt::Throw(Throw {
            value: Box::new(value),
            token,
        })
    }

    /// Creates a new function (`Fn`) statement.
    ///
    /// # Arguments
    ///
    /// * `fn_token` - The token representing the `fn` keyword.
    /// * `name` - The name of the function.
    /// * `params` - A vector of function parameters.
    /// * `body` - The block of code representing the function body.
    /// * `public` - A boolean indicating if the function is public.
    /// * `return_type` - An optional return type annotation.
    /// * `is_static` - A boolean indicating if the function is static.
    ///
    /// # Returns
    ///
    /// A `Stmt::Fn` variant containing the provided function details.
    pub fn new_fn(
        fn_token: Token,
        name: String,
        params: Vec<FnParam>,
        body: Block,
        public: bool,
        return_type: Option<TypeAnnotation>,
        is_static: bool,
    ) -> Self {
        Stmt::Fn(Fn {
            fn_token,
            name,
            params,
            body,
            public,
            return_type,
            is_static,
        })
    }

    /// Creates a new `Use` statement.
    ///
    /// # Arguments
    ///
    /// * `use_token` - The token representing the `use` keyword.
    /// * `from` - The token representing the module or path to import from.
    /// * `items` - A vector of tokens representing the items to import.
    ///
    /// # Returns
    ///
    /// A `Stmt::Use` variant containing the provided import details.
    pub fn new_use(use_token: Token, from: Token, items: Vec<Token>) -> Self {
        Stmt::Use(Use {
            use_token,
            from,
            items,
        })
    }

    /// Creates a new `If` statement.
    ///
    /// # Arguments
    ///
    /// * `if_token` - The token representing the `if` keyword.
    /// * `condition` - The expression to evaluate as the condition.
    /// * `then_block` - The block of code to execute if the condition is true.
    /// * `else_ifs` - A vector of `ElseBlock` representing `else if` clauses.
    /// * `else_block` - An optional `ElseBlock` representing the `else` clause.
    ///
    /// # Returns
    ///
    /// A `Stmt::If` variant containing the provided condition and blocks.
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

    /// Creates a new `Let` statement.
    ///
    /// # Arguments
    ///
    /// * `ident` - The token representing the variable identifier.
    /// * `initializer` - The expression used to initialize the variable.
    /// * `type_annotation` - An optional type annotation for the variable.
    ///
    /// # Returns
    ///
    /// A `Stmt::Let` variant containing the provided variable details.
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

    /// Creates a new `Return` statement.
    ///
    /// # Arguments
    ///
    /// * `return_token` - The token representing the `return` keyword.
    /// * `expr` - An optional expression to return.
    ///
    /// # Returns
    ///
    /// A `Stmt::Return` variant containing the provided return value.
    pub fn new_return(return_token: Token, expr: Option<Box<Expr>>) -> Self {
        Stmt::Return(Return { return_token, expr })
    }

    /// Creates a new `Struct` statement.
    ///
    /// # Arguments
    /// * `struct_token` - The token representing the `struct` keyword.
    /// * `name` - The name of the struct.
    ///
    /// # Returns
    /// A `Stmt::Struct` variant containing the provided struct details.
    pub fn new_struct(
        struct_token: Token,
        name: Token,
        fields: IndexMap<String, StructField>,
        public: bool,
    ) -> Self {
        Stmt::Struct(Struct {
            struct_token,
            name,
            fields,
            public,
            impls: vec![],
            trait_impls: vec![],
        })
    }

    /// Creates a new `Const` statement.
    ///
    /// # Arguments
    /// * `expr` - The expression to assign to the constant.
    /// * `ident` - The identifier token for the constant.
    /// * `public` - A boolean indicating if the constant is public.
    ///
    /// # Returns
    /// A `Stmt::Const` variant containing the provided constant details.
    pub fn new_const(expr: Box<Expr>, ident: Token, public: bool) -> Self {
        Stmt::Const(Const {
            expr,
            ident,
            public,
        })
    }

    /// Creates a new `TraitDef` statement.
    ///
    /// # Arguments
    /// * `trait_token` - The token representing the `trait` keyword.
    /// * `name` - The name of the trait.
    /// * `methods` - A vector of function declarations representing the trait methods.
    pub fn new_trait_def(trait_token: Token, name: Token, methods: Vec<Fn>, public: bool) -> Self {
        Stmt::TraitDef(TraitDef {
            trait_token,
            name,
            methods,
            public,
        })
    }

    /// Creates a new `StructImpl` statement.
    ///
    /// # Arguments
    /// * `impl_token` - The token representing the `impl` keyword.
    /// * `struct_name` - The name of the struct being implemented.
    /// * `methods` - A vector of function declarations representing the struct methods.
    ///
    /// # Returns
    /// A `Stmt::StructImpl` variant containing the provided struct implementation details.
    pub fn new_struct_impl(impl_token: Token, struct_name: Token, methods: Vec<Fn>) -> Self {
        Stmt::StructImpl(StructImpl {
            impl_token,
            struct_name,
            methods,
        })
    }

    /// Creates a new `TraitImpl` statement.
    ///
    /// # Arguments
    /// * `impl_token` - The token representing the `impl` keyword.
    /// * `trait_name` - The name of the trait being implemented.
    /// * `for_token` - The token representing the `for` keyword.
    /// * `struct_name` - The name of the struct implementing the trait.
    /// * `methods` - A vector of function declarations representing the trait methods.
    ///
    /// # Returns
    /// A `Stmt::TraitImpl` variant containing the provided trait implementation details.
    pub fn new_trait_impl(
        impl_token: Token,
        trait_name: Token,
        for_token: Token,
        struct_name: Token,
        methods: Vec<Fn>,
    ) -> Self {
        Stmt::TraitImpl(TraitImpl {
            impl_token,
            trait_name,
            for_token,
            struct_name,
            methods,
        })
    }
}

impl Stmt {
    /// Retrieves a reference to the function (`Fn`) contained within the statement.
    ///
    /// # Panics
    ///
    /// Panics if the statement is not a `Fn` variant.
    ///
    /// # Returns
    ///
    /// A reference to the contained `Fn` struct.
    pub fn as_function(&self) -> &Fn {
        match self {
            Stmt::Fn(f) => f,
            _ => panic!("Expected function"),
        }
    }
}

/// Represents a function parameter in the AST.
///
/// Each parameter has an identifier, an optional type annotation, and a flag indicating
/// whether it is a rest parameter (e.g., `...args`).
#[derive(Clone, Debug, PartialEq)]
pub struct FnParam {
    /// The token representing the parameter identifier.
    pub ident: Token,
    /// The type annotation of the parameter.
    pub type_annotation: TypeAnnotation,
    /// Indicates whether the parameter is a rest parameter.
    pub is_rest: bool,
}

impl GetSpan for FnParam {
    fn span(&self) -> TextSpan {
        TextSpan::combine(vec![self.ident.span.clone(), self.type_annotation.span()]).unwrap()
    }
}

impl FnParam {
    /// Creates a new function parameter.
    ///
    /// # Arguments
    ///
    /// * `ident` - The token representing the parameter identifier.
    /// * `type_annotation` - The type annotation of the parameter.
    /// * `is_rest` - A boolean indicating if the parameter is a rest parameter.
    ///
    /// # Returns
    ///
    /// A new `FnParam` instance.
    pub fn new(ident: Token, type_annotation: TypeAnnotation, is_rest: bool) -> Self {
        Self {
            ident,
            type_annotation,
            is_rest,
        }
    }
}

/// Represents a type annotation in the AST.
///
/// A type annotation consists of a colon and the type name.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAnnotation {
    /// The token representing the colon (`:`) or arrow (`->`) separator.
    pub separator: Option<Token>,
    /// The token representing the type name.
    pub token_name: Option<Token>,
    /// Type name
    pub kind: TypeKind,
    /// Is nullable?
    pub is_nullable: bool,
    /// Stores id to module for type
    pub module_id: Option<String>,
    /// Generic type name
    pub generics: Vec<TypeAnnotation>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    String,
    Int,
    Float,
    Char,
    Bool,
    Vec,
    Object,
    Anytype,
    Void,
    Custom(String),
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeKind::String => write!(f, "string"),
            TypeKind::Int => write!(f, "int"),
            TypeKind::Float => write!(f, "float"),
            TypeKind::Char => write!(f, "char"),
            TypeKind::Bool => write!(f, "bool"),
            TypeKind::Vec => write!(f, "vec"),
            TypeKind::Object => write!(f, "object"),
            TypeKind::Anytype => write!(f, "anytype"),
            TypeKind::Void => write!(f, "void"),
            TypeKind::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl TypeKind {
    pub fn from_str(s: &str) -> Self {
        match s {
            "string" => TypeKind::String,
            "int" => TypeKind::Int,
            "float" => TypeKind::Float,
            "char" => TypeKind::Char,
            "bool" => TypeKind::Bool,
            "vec" => TypeKind::Vec,
            "object" => TypeKind::Object,
            "anytype" => TypeKind::Anytype,
            "void" => TypeKind::Void,
            _ => TypeKind::Custom(s.to_string()),
        }
    }
}

impl TypeAnnotation {
    pub fn is_any(&self) -> bool {
        self.kind == TypeKind::Anytype
    }

    pub fn is_generic(&self) -> bool {
        self.generics.len() > 0
    }
    
    pub fn match_generic(&self, generic: TypeKind, args: Vec<TypeKind>) -> bool {
        let generics_names = self
            .generics
            .iter()
            .map(|g| g.kind.clone())
            .collect::<Vec<TypeKind>>();

        self.kind == generic && generics_names == args
    }
}

impl GetSpan for TypeAnnotation {
    fn span(&self) -> TextSpan {
        let mut spans = vec![];

        if let Some(token) = &self.separator {
            spans.push(token.span.clone());
        }

        if let Some(token) = &self.token_name {
            spans.push(token.span.clone());
        }

        for generic in &self.generics {
            spans.push(generic.span());
        }

        TextSpan::combine(spans).unwrap()
    }
}

/// Represents a function declaration in the AST.
///
/// A function includes its name, parameters, body, export status, and an optional return type.
#[derive(Clone, PartialEq, Debug)]
pub struct Fn {
    /// The token representing the `fn` keyword.
    pub fn_token: Token,
    /// The name of the function.
    pub name: String,
    /// The list of parameters for the function.
    pub params: Vec<FnParam>,
    /// The body of the function as a block of statements.
    pub body: Block,
    /// Indicates whether the function is public.
    pub public: bool,
    /// An optional return type annotation.
    pub return_type: Option<TypeAnnotation>,
    /// Indicates whether the function is static.
    pub is_static: bool,
}

/// Represents an `if` statement in the AST.
///
/// An `if` statement includes a condition, a `then` block, and optional `else if` and `else` blocks.
#[derive(Clone, Debug, PartialEq)]
pub struct If {
    /// The token representing the `if` keyword.
    pub if_token: Token,
    /// The condition expression to evaluate.
    pub condition: Box<Expr>,
    /// The block of code to execute if the condition is true.
    pub then_block: Block,
    /// A list of `else if` blocks.
    pub else_ifs: Vec<ElseBlock>,
    /// An optional `else` block.
    pub else_block: Option<ElseBlock>,
}

/// Represents an `else` or `else if` block in the AST.
///
/// An `ElseBlock` can optionally include a condition (for `else if`) and contains a block of statements.
#[derive(Clone, Debug, PartialEq)]
pub struct ElseBlock {
    /// The condition expression for an `else if` block. `None` for a plain `else` block.
    pub condition: Box<Expr>,
    /// The block of code to execute for this `else if` or `else` block.
    pub block: Block,
    /// Indicates whether this block is an `else if` (`true`) or a plain `else` (`false`).
    pub else_if: bool,
}

/// Represents a `use` statement for importing modules or items in the AST.
///
/// A `use` statement specifies the source module and the items to import.
#[derive(Clone, Debug, PartialEq)]
pub struct Use {
    /// The token representing the `use` keyword.
    pub use_token: Token,
    /// The token representing the module or path to import from.
    pub from: Token,
    /// A list of tokens representing the items to import.
    pub items: Vec<Token>,
}

/// Represents a block of statements enclosed in braces in the AST.
///
/// A `Block` contains a sequence of statements that are executed together.
#[derive(Clone, PartialEq)]
pub struct Block {
    /// The list of statements contained within the block.
    pub stmts: Vec<Stmt>,
}

impl Debug for Block {
    /// Custom implementation of the `Debug` trait for the `Block` struct.
    ///
    /// This provides a formatted debug representation, displaying the number of statements.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Block")
            .field("stmts", &self.stmts.len())
            .finish()
    }
}

/// Represents a `return` statement in the AST.
///
/// A `return` statement exits a function, optionally returning an expression.
#[derive(Clone, PartialEq, Debug)]
pub struct Return {
    /// The token representing the `return` keyword.
    pub return_token: Token,
    /// An optional expression to return from the function.
    pub expr: Option<Box<Expr>>,
}
