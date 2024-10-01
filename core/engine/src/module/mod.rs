use std::fmt::Debug;
use std::path::{Path, PathBuf};
use roan_ast::source::{Source};
use anyhow::Result;
use log::debug;
use roan_ast::{Lexer, Parser, Token, Fn, Let, Stmt, Use, Ast};
use roan_error::error::PulseError::{ImportError, ModuleNotFoundError};
use crate::context::Context;

pub mod loader;

#[derive(Debug, Clone)]
pub enum ExportType {
    Function(Fn),
    Variable,
}

#[derive(Clone, Debug)]
pub struct Module {
    source: Source,
    path: Option<PathBuf>,
    tokens: Vec<Token>,
    ast: Ast,
    functions: Vec<Fn>,
    exports: Vec<(String, ExportType)>,
    imports: Vec<Use>,
    variables: Vec<Let>,
}

impl Module {
    /// Creates a new `Module` from the specified `Source`.
    ///
    /// # Parameters
    /// - `source` - The source of the module.
    ///
    /// # Returns
    /// The new `Module`.
    ///
    /// # Examples
    /// ```rust
    /// use roan_engine::module::Module;
    /// use roan_ast::source::Source;
    /// use roan_engine::context::Context;
    /// let source = Source::from_bytes("fn main() { }");
    /// let module = Module::new(source);
    /// ```
    pub fn new(source: Source) -> Self {
        let path = source.path().as_deref().map(Path::to_path_buf);

        Self { source, path, tokens: vec![], functions: vec![], exports: vec![], imports: vec![], variables: vec![], ast: Ast::new() }
    }

    /// Returns the path of the module.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Returns the source of the module.
    pub fn source(&self) -> &Source {
        &self.source
    }

    /// Parses the module.
    ///
    /// First, the module is lexed into tokens. Then, the tokens are parsed into an AST.
    pub fn parse(&mut self) -> Result<()> {
        debug!("Parsing module from source");
        let mut lexer = Lexer::new(self.source.clone());

        let tokens = lexer.lex()?;
        debug!("Parsed {} tokens", tokens.len());
        self.tokens = tokens;

        let mut parser = Parser::new(self.tokens.clone());

        debug!("Parsing tokens into AST");

        let ast = parser.parse()?;

        self.ast = ast;

        Ok(())
    }

    pub fn interpret(&mut self, ctx: &Context) -> Result<()> {
        for stmt in self.ast.stmts.clone() {
            self.interpret_stmt(stmt, ctx)?;
        }

        Ok(())
    }

    /// Interpret statement from the module.
    pub fn interpret_stmt(&mut self, stmt: Stmt, ctx: &Context) -> Result<()> {
        match stmt {
            Stmt::Fn(f) => {
                debug!("Interpreting function: {}", f.name);
                self.functions.push(f.clone());

                if f.exported {
                    self.exports.push((f.name.clone(), ExportType::Function(f.clone())));
                }

                for stmt in f.body.stmts {
                    self.interpret_stmt(stmt, ctx)?;
                }
            }
            Stmt::Use(u) => {
                debug!("Interpreting use: {}", u.from.literal());
                let mut module = ctx.module_loader.load(&self, &u.from.literal(), ctx)
                    .map_err(|e| ModuleNotFoundError(u.from.literal(), u.from.span.clone()))?;
                module.parse()?;
                module.interpret(ctx)?;

                let imported_items: Vec<(String, &Token)> = u.items.iter().map(|i| (i.literal(), i)).collect::<Vec<_>>();

                for (name, item) in imported_items {
                    match module.find_function(&name) {
                        Some(f) => {
                            self.functions.push(f.clone());
                        }
                        None => Err(ImportError(name, item.span.clone()))?,
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Looks for a function with the specified name.
    pub fn find_function(&self, name: &str) -> Option<&Fn> {
        debug!("Looking for function: {}", name);
        self.functions.iter().find(|f| f.name == name)
    }

    /// Looks for a variable with the specified name.
    pub fn find_variable(&self, name: &str) -> Option<&Let> {
        debug!("Looking for variable: {}", name);
        self.variables.iter().find(|v| v.ident.literal() == name)
    }
}
