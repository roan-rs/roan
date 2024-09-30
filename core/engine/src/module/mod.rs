use std::path::{Path, PathBuf};
use roan_ast::source::{Source};
use anyhow::Result;
use log::debug;
use roan_ast::{Lexer, Token};

pub mod fs;
pub mod loader;

#[derive(Clone, Debug)]
pub struct Module {
    source: Source,
    path: Option<PathBuf>,
    tokens: Vec<Token>,
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
    /// let source = Source::from_bytes("fn main() { }".as_bytes().into_vec());
    ///
    /// let module = Module::new(source);
    /// ```
    pub fn new(source: Source) -> Self {
        let path = source.path().as_deref().map(Path::to_path_buf);

        Self { source, path, tokens: vec![] }
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

        // Create a Lexer by borrowing the Source
        let mut lexer = Lexer::new(self.source.clone());

        // Perform lexing
        let tokens = lexer.lex()?;

        debug!("Parsed {} tokens", tokens.len());

        // Store the tokens in the module
        self.tokens = tokens;

        Ok(())
    }
}
