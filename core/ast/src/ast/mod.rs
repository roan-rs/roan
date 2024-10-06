use roan_error::TextSpan;

/// Modules that contain definitions and code for statements and expressions in the AST.
pub mod statements;
pub mod expr;

/// Makes items from `statements` and `expr` modules available for use with the AST.
pub use statements::*;
pub use expr::*;

/// Represents the Abstract Syntax Tree (AST) for the language.
///
/// The AST is a structured view of the code, with each part of the code represented as a "node."
/// It is used for understanding and analyzing the code, such as checking for errors or generating final output.
///
/// # Examples
///
/// ```rust
/// use roan_ast::Ast;
/// let mut ast = Ast::new();
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Ast {
    /// A list of top-level statements (instructions) in the AST.
    pub stmts: Vec<Stmt>,
}

impl Ast {
    /// Creates an empty AST with no statements.
    ///
    /// # Returns
    /// A new `Ast` with an empty list of statements.
    pub fn new() -> Self {
        Self { stmts: vec![] }
    }

    /// Adds a statement to the AST.
    ///
    /// # Arguments
    /// * `stmt` - The statement to add.
    pub fn add_stmt(&mut self, stmt: Stmt) {
        self.stmts.push(stmt);
    }

    /// Returns all statements in the AST.
    ///
    /// # Returns
    /// A reference to the list of statements.
    pub fn statements(&self) -> &Vec<Stmt> {
        &self.stmts
    }
}

/// A trait to get the source code position (span) of a node in the AST.
///
/// `GetSpan` helps retrieve the part of the code (location) related to a node in the AST.
/// This is useful for showing where errors happen or for debugging.
pub trait GetSpan {
    /// Returns the `TextSpan` that shows where this AST node is in the source code.
    fn span(&self) -> TextSpan;
}
