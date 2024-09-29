use anyhow::Result;
use roan_error::error::PulseError::InvalidToken;
use roan_error::position::Position;
use roan_error::span::TextSpan;
use crate::lexer::token::{Token, TokenKind};

pub mod token;

pub struct Lexer {
    pub source: String,
    pub tokens: Vec<Token>,
    pub position: Position,
}

impl Lexer {
    pub fn from_source(source: String) -> Self {
        Self {
            source,
            tokens: vec![],
            position: Position::new(0, 0, 0),
        }
    }
}

impl Lexer {
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

    pub fn is_eof(&self) -> bool {
        self.position.index >= self.source.len()
    }

    pub fn current(&mut self) -> Option<char> {
        self.source.chars().nth(self.position.index)
    }

    pub fn consume(&mut self) -> Option<char> {
        if self.position.index >= self.source.len() {
            return None;
        }
        let c = self.current();

        self.update_position(c?);

        c
    }

    fn update_position(&mut self, c: char) {
        if c == '\n' {
            self.position.line += 1;
            self.position.column = 0;
        } else {
            self.position.column += 1;
        }
        self.position.index += 1;
    }

    // can be made of letters, digits, and underscores
    pub fn is_identifier_start(&self, c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    pub fn is_number_start(&self, c: char) -> bool {
        c.is_digit(10)
    }

    pub fn peek(&self) -> Option<char> {
        if self.position.index + 1 >= self.source.len() {
            None
        } else {
            self.source.chars().nth(self.position.index + 1)
        }
    }

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
            let literal = self.source[start_pos.index..end_pos.index].to_string();
            Ok(Some(Token::new(
                kind,
                TextSpan::new(start_pos, end_pos, literal),
            )))
        } else {
            Ok(None)
        }
    }

    pub fn match_next(&mut self, ch: char) -> bool {
        if let Some(c) = self.current() {
            if c == ch {
                return true;
            }
        }
        false
    }

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

    // Can be float or integer
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

#[derive(Debug)]
pub enum NumberType {
    Integer,
    Float,
}
