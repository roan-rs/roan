use anyhow::Result;
use std::{fs, path::PathBuf};
use roan_engine::{Token, Lexer, Parser};

#[derive(Debug)]
pub struct Instance {
    pub path: PathBuf,
    pub tokens: Vec<Token>,
    pub content: String,
}

impl Instance {
    pub fn from_path(path: PathBuf) -> Instance {
        Instance {
            path,
            tokens: vec![],
            content: String::new(),
        }
    }
}

impl Instance {
    pub fn interpret(&mut self) -> Result<()> {
        let main_content = fs::read_to_string(self.path.clone())?;
        self.content = main_content.clone();

        let mut lexer = Lexer::from_source(main_content);
        let tokens = lexer.lex()?;

        self.tokens = tokens;

        let mut parser = Parser::new(self.tokens.clone());
        let ast = parser.parse()?;

        Ok(())
    }
}
