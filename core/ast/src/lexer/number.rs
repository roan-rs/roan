use crate::{Lexer, TokenKind};
use anyhow::Result;

/// The type of number.
#[derive(Debug)]
pub enum NumberType {
    Integer(i64),
    Float(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberLiteral {}

impl NumberLiteral {
    pub fn lex_number(lexer: &mut Lexer, c: char) -> Result<TokenKind> {
        let mut number = NumberType::Integer(0);

        lexer.consume();

        if c == '0' {
            if let Some(c) = lexer.consume() {
                match c {
                    'x' | 'X' => {
                        // Hexadecimal
                        let mut hex = String::new();
                        while let Some(c) = lexer.current() {
                            if c.is_digit(16) {
                                hex.push(c);
                            } else {
                                break;
                            }
                            lexer.consume();
                        }

                        number = NumberType::Integer(i64::from_str_radix(&hex, 16)?);
                    }
                    'o' | 'O' => {
                        // Octal
                        let mut oct = String::new();
                        while let Some(c) = lexer.current() {
                            if c.is_digit(8) {
                                oct.push(c);
                            } else {
                                break;
                            }
                            lexer.consume();
                        }

                        number = NumberType::Integer(i64::from_str_radix(&oct, 8)?);
                    }
                    'b' | 'B' => {
                        // Binary
                        let mut bin = String::new();
                        while let Some(c) = lexer.current() {
                            if c == '0' || c == '1' {
                                bin.push(c);
                            } else {
                                break;
                            }
                            lexer.consume();
                        }

                        number = NumberType::Integer(i64::from_str_radix(&bin, 2)?);
                    }
                    c if c.is_digit(10) => {
                        let mut num_str = String::from("0");
                        num_str.push(c);
                        while let Some(c) = lexer.current() {
                            if c.is_digit(10) {
                                num_str.push(c);
                                lexer.consume();
                            } else {
                                break;
                            }
                        }

                        number = NumberType::Integer(num_str.parse()?);
                    }
                    _ => {
                        return Ok(TokenKind::Integer(0));
                    }
                }
            }
        } else {
            let mut num_str = String::new();
            num_str.push(c);

            while let Some(c) = lexer.current() {
                if c.is_digit(10) {
                    num_str.push(c);
                    lexer.consume();
                } else if c == '.' {
                    num_str.push(c);
                    lexer.consume();
                    while let Some(c) = lexer.current() {
                        if c.is_digit(10) {
                            num_str.push(c);
                            lexer.consume();
                        } else {
                            break;
                        }
                    }
                    number = NumberType::Float(num_str.parse()?);
                    break;
                } else {
                    break;
                }
            }

            if let NumberType::Integer(_) = number {
                number = NumberType::Integer(num_str.parse()?);
            }
        }

        Ok(match number {
            NumberType::Integer(i) => TokenKind::Integer(i),
            NumberType::Float(f) => TokenKind::Float(f),
        })
    }
}
