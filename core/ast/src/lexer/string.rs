use crate::{Lexer, TokenKind};
use anyhow::Result;
use roan_error::{error::RoanError::InvalidEscapeSequence, TextSpan};

#[derive(Debug, Clone, PartialEq)]
pub struct StringLiteral {}

impl StringLiteral {
    pub fn lex_string(lexer: &mut Lexer) -> Result<TokenKind> {
        let string = StringLiteral::consume_string(lexer)?;
        Ok(TokenKind::String(string))
    }

    pub fn consume_string(lexer: &mut Lexer) -> Result<String> {
        let mut str = String::new();

        lexer.consume();

        while let Some(c) = lexer.current() {
            if c == '"' {
                lexer.consume();
                break;
            }

            if c == '\\' {
                lexer.consume();
                if let Some(next) = lexer.current() {
                    match next {
                        'n' => str.push('\n'),
                        'r' => str.push('\r'),
                        't' => str.push('\t'),
                        '\\' => str.push('\\'),
                        '"' => str.push('"'),
                        _ => {
                            return Err(InvalidEscapeSequence(
                                next.to_string(),
                                TextSpan::new(lexer.position, lexer.position, next.to_string()),
                            )
                            .into())
                        }
                    }
                    lexer.consume();
                }
            } else {
                str.push(c);
                lexer.consume();
            }
        }

        Ok(str)
    }
}
