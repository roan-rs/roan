use crate::lexer::token::{Token, TokenKind};
use anyhow::Result;
use roan_error::{error::PulseError::InvalidToken, position::Position, span::TextSpan};
use crate::source::Source;

pub mod token;

/// The lexer is responsible for converting the source code into a list of tokens.
pub struct Lexer {
    pub source: Source,
    pub tokens: Vec<Token>,
    pub position: Position,
}

impl Lexer {
    /// Create a new lexer from a source string.
    ///
    /// # Arguments
    /// - `source` - A string slice containing the source code.
    ///
    /// # Example
    /// ```rust
    /// use roan_ast::{Lexer, TokenKind};
    /// use roan_ast::source::Source;
    /// let source = Source::from_string("let x = 10;".to_string());
    /// let mut lexer = Lexer::new(source);
    /// let tokens = lexer.lex().expect("Failed to lex source code");
    ///
    /// assert_eq!(tokens.first().unwrap().kind, TokenKind::Let);
    /// ```
    /// ```
    pub fn new(source: Source) -> Self {
        Self {
            source,
            tokens: vec![],
            position: Position::new(0, 0, 0),
        }
    }
}

impl Lexer {
    /// Lex the source code and return a list of tokens.
    ///
    /// During the lexing process, the lexer will consume the source code character by character
    /// and convert it into a list of tokens. The lexer will skip whitespace and comments.
    ///
    /// When EOF is reached, the lexer will return the list of tokens.
    pub fn lex(&mut self) -> Result<Vec<Token>> {
        while let Some(token) = self.next_token()? {
            if token.kind == TokenKind::Whitespace {
                continue;
            }
            if token.kind == TokenKind::EOF {
                self.tokens.push(token);
                break;
            }
            self.tokens.push(token);
        }

        Ok(self.tokens.clone())
    }

    /// Check if the lexer has reached the end of the source code.
    pub fn is_eof(&self) -> bool {
        self.position.index >= self.source.len()
    }

    /// Get the current character in the source code.
    pub fn current(&mut self) -> Option<char> {
        self.source.chars().nth(self.position.index)
    }

    /// Consume the current character and move to the next one.
    pub fn consume(&mut self) -> Option<char> {
        if self.position.index >= self.source.len() {
            return None;
        }
        let c = self.current();

        self.update_position(c?);

        c
    }

    /// Update the position of the lexer.
    ///
    /// The position is updated based on the current character.
    /// The position includes the line, column, and index of the character.
    ///
    /// If the character is a newline, the line is incremented and the column is reset to 0.
    fn update_position(&mut self, c: char) {
        if c == '\n' {
            self.position.line += 1;
            self.position.column = 0;
        } else {
            self.position.column += 1;
        }
        self.position.index += 1;
    }

    /// Check if the character is a valid identifier start character.
    pub fn is_identifier_start(&self, c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    /// Check if the character is a valid number start character.
    pub fn is_number_start(&self, c: char) -> bool {
        c.is_digit(10)
    }

    /// Peek at the next character in the source code.
    pub fn peek(&self) -> Option<char> {
        if self.position.index + 1 >= self.source.len() {
            None
        } else {
            self.source.chars().nth(self.position.index + 1)
        }
    }

    /// Parse a string literal.
    pub fn parse_string(&mut self) -> Result<String> {
        let mut str = String::new();

        self.consume();

        while let Some(c) = self.current() {
            if c == '"' {
                self.consume();
                break;
            }

            if c == '\\' {
                self.consume();
                if let Some(next) = self.current() {
                    match next {
                        'n' => str.push('\n'),
                        'r' => str.push('\r'),
                        't' => str.push('\t'),
                        '\\' => str.push('\\'),
                        '"' => str.push('"'),
                        _ => return Err(anyhow::anyhow!("Invalid escape character")),
                    }
                    self.consume();
                }
            } else {
                str.push(c);
                self.consume();
            }
        }

        Ok(str)
    }

    /// Get the next token in the source code.
    pub fn next_token(&mut self) -> Result<Option<Token>> {
        if let Some(c) = self.current() {
            let start_pos = self.position;
            let kind = if c.is_whitespace() {
                while let Some(c) = self.current() {
                    if !c.is_whitespace() {
                        break;
                    }
                    self.consume();
                }
                TokenKind::Whitespace
            } else if c == '"' {
                let string = self.parse_string()?;
                TokenKind::String(string)
            } else if c.is_numeric() {
                let number = self.consume_number();
                match number.0 {
                    NumberType::Integer => TokenKind::Integer(number.1.parse()?),
                    NumberType::Float => TokenKind::Float(number.1.parse()?),
                }
            } else if self.is_identifier_start(c) {
                let ident = self.consume_identifier();
                match ident.as_str() {
                    "fn" => TokenKind::Fn,
                    "let" => TokenKind::Let,
                    "if" => TokenKind::If,
                    "else" => TokenKind::Else,
                    "return" => TokenKind::Return,
                    "true" => TokenKind::True,
                    "false" => TokenKind::False,
                    "null" => TokenKind::Null,
                    "while" => TokenKind::While,
                    "for" => TokenKind::For,
                    "in" => TokenKind::In,
                    "break" => TokenKind::Break,
                    "continue" => TokenKind::Continue,
                    "use" => TokenKind::Use,
                    "export" => TokenKind::Export,
                    "from" => TokenKind::From,

                    _ => TokenKind::Identifier,
                }
            } else {
                let punc = match c {
                    '(' => TokenKind::LeftParen,
                    ')' => TokenKind::RightParen,
                    '{' => TokenKind::LeftBrace,
                    '}' => TokenKind::RightBrace,
                    '[' => TokenKind::LeftBracket,
                    ']' => TokenKind::RightBracket,
                    ',' => TokenKind::Comma,
                    '.' => TokenKind::Dot,
                    ':' => TokenKind::Colon,
                    ';' => TokenKind::Semicolon,
                    '/' => {
                        if self.match_next('/') {
                            while let Some(c) = self.current() {
                                if c == '\n' {
                                    break;
                                }
                                self.consume();
                            }
                            TokenKind::Comment
                        } else {
                            TokenKind::Slash
                        }
                    }
                    '+' => {
                        self.consume();
                        if self.match_next('+') {
                            TokenKind::Increment
                        } else if self.match_next('=') {
                            TokenKind::PlusEquals
                        } else {
                            TokenKind::Plus
                        }
                    }
                    '-' => {
                        self.consume();
                        if self.match_next('-') {
                            TokenKind::Decrement
                        } else if self.match_next('=') {
                            TokenKind::MinusEquals
                        } else if self.match_next('>') {
                            TokenKind::Arrow
                        } else {
                            TokenKind::Minus
                        }
                    }
                    '*' => {
                        self.consume();
                        if self.match_next('*') {
                            TokenKind::DoubleAsterisk
                        } else {
                            TokenKind::Asterisk
                        }
                    }
                    '%' => TokenKind::Percent,
                    '^' => TokenKind::Caret,
                    '!' => {
                        self.consume();
                        if self.match_next('=') {
                            TokenKind::BangEquals
                        } else {
                            TokenKind::Bang
                        }
                    }
                    '=' => {
                        self.consume();
                        if self.match_next('=') {
                            TokenKind::EqualsEquals
                        } else {
                            TokenKind::Equals
                        }
                    }
                    '<' => {
                        self.consume();
                        if self.match_next('=') {
                            TokenKind::LessThanEquals
                        } else {
                            TokenKind::LessThan
                        }
                    }
                    '>' => {
                        self.consume();
                        if self.match_next('=') {
                            TokenKind::GreaterThanEquals
                        } else {
                            TokenKind::GreaterThan
                        }
                    }
                    '&' => {
                        self.consume();
                        if self.match_next('&') {
                            TokenKind::And
                        } else {
                            TokenKind::Ampersand
                        }
                    }
                    '|' => {
                        self.consume();
                        if self.match_next('|') {
                            TokenKind::Or
                        } else {
                            TokenKind::Pipe
                        }
                    }
                    _ => {
                        self.consume();
                        return Err(InvalidToken(
                            c.to_string(),
                            TextSpan::new(start_pos, self.position, c.to_string()),
                        )
                            .into());
                    }
                };

                self.consume();
                punc
            };

            let end_pos = self.position;
            let literal = self.source.get_between(start_pos.index, end_pos.index);
            Ok(Some(Token::new(
                kind,
                TextSpan::new(start_pos, end_pos, literal),
            )))
        } else {
            Ok(None)
        }
    }

    /// Check if the next character matches the given character.
    pub fn match_next(&mut self, ch: char) -> bool {
        if let Some(c) = self.current() {
            if c == ch {
                return true;
            }
        }
        false
    }

    /// Consume an identifier.
    pub fn consume_identifier(&mut self) -> String {
        let mut ident = String::new();

        while let Some(c) = self.current() {
            if self.is_identifier_start(c) {
                ident.push(c);
            } else {
                break;
            }
            self.consume();
        }

        ident
    }

    /// Consume a number.
    ///
    /// Can be either an integer or a float.
    pub fn consume_number(&mut self) -> (NumberType, String) {
        let mut number = String::new();
        let mut number_type = NumberType::Integer;

        while let Some(c) = self.current() {
            if c.is_digit(10) {
                number.push(c);
            } else if c == '.' {
                number.push(c);
                number_type = NumberType::Float;
            } else {
                break;
            }
            self.consume();
        }

        (number_type, number)
    }
}

/// The type of number.
#[derive(Debug)]
pub enum NumberType {
    Integer,
    Float,
}