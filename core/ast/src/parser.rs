use crate::{
    ast::{
        Ast, BinOpAssociativity, BinOpKind, BinOperator, Block, ElseBlock, Expr, FnParam,
        FunctionType, Stmt, TypeAnnotation, UnOpKind, UnOperator,
    },
    lexer::token::{Token, TokenKind},
};
use anyhow::Result;
use log::debug;
use roan_error::error::PulseError::{ExpectedToken, MultipleRestParameters, RestParameterNotLastPosition, UnexpectedToken};

/// Struct responsible for parsing the tokens into an AST
#[derive(Debug)]
pub struct Parser {
    /// The tokens to parse from the lexer
    pub tokens: Vec<Token>,
    /// The current token index
    pub current: usize,
}

impl Parser {
    /// Creates a new parser with the given tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }
}

impl Parser {
    /// Parses the tokens into an AST until the last token or EOF
    pub fn parse(&mut self) -> Result<Ast> {
        let mut ast = Ast::new();

        while !self.is_eof() {
            let stmt = self.parse_stmt()?;

            if let Some(stmt) = stmt {
                ast.stmts.push(stmt);
            }
        }

        Ok(ast)
    }

    /// Consumes the current token and returns the previous token
    pub fn consume(&mut self) -> Token {
        if !self.is_eof() {
            self.current += 1;
        }

        self.previous()
    }

    /// Returns the previous token
    pub fn previous(&self) -> Token {
        assert!(self.current > 0);

        self.tokens[self.current - 1].clone()
    }

    /// Returns the next token
    pub fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    pub fn peek_next(&self) -> Token {
        self.tokens[self.current + 1].clone()
    }

    pub fn is_eof(&self) -> bool {
        self.current >= self.tokens.len() || self.peek().kind == TokenKind::EOF
    }

    /// Consumes the current token if it is of the given kind
    pub fn possible_check(&mut self, kind: TokenKind) {
        if self.peek().kind == kind {
            self.consume();
        }
    }

    /// Expects the current token to be of the given kind and then consumes it or throws an error
    pub fn expect(&mut self, kind: TokenKind) -> Result<Token> {
        let token = self.peek();

        if token.kind == kind {
            Ok(self.consume())
        } else {
            Err(ExpectedToken(
                kind.to_string(),
                format!("Expected token of kind: {}", kind),
                token.span.clone(),
            )
                .into())
        }
    }
}

impl Parser {
    /// Parses a statement from the tokens
    pub fn parse_stmt(&mut self) -> Result<Option<Stmt>> {
        let token = self.peek();

        let stmt = match token.kind {
            TokenKind::Fn | TokenKind::Export => Some(self.parse_fn()?),
            TokenKind::Use => Some(self.parse_use()?),
            TokenKind::If => Some(self.parse_if()?),
            TokenKind::Let => Some(self.parse_let()?),
            TokenKind::Throw => Some(self.parse_throw()?),
            TokenKind::Try => Some(self.parse_try()?),
            TokenKind::LeftBrace => {
                self.consume();
                let block = self.parse_block()?;
                self.expect(TokenKind::RightBrace)?;
                Some(Stmt::Block(block))
            }
            TokenKind::Return => self.parse_return()?,
            TokenKind::Semicolon => {
                self.consume();
                None
            }
            _ => Some(self.expression_stmt()?),
        };

        Ok(stmt)
    }

    pub fn parse_throw(&mut self) -> Result<Stmt> {
        debug!("Parsing throw statement");
        let throw_token = self.consume();
        let value = self.parse_expr()?;

        self.possible_check(TokenKind::Semicolon);

        Ok(Stmt::new_throw(throw_token, value))
    }

    pub fn parse_try(&mut self) -> Result<Stmt> {
        debug!("Parsing try statement");
        let try_token = self.consume();

        self.expect(TokenKind::LeftBrace)?;
        let try_block = self.parse_block()?;
        self.expect(TokenKind::RightBrace)?;

        self.expect(TokenKind::Catch)?;

        let error_ident = self.expect(TokenKind::Identifier)?;

        self.expect(TokenKind::LeftBrace)?;
        let catch_block = self.parse_block()?;
        self.expect(TokenKind::RightBrace)?;

        Ok(Stmt::new_try(try_token, try_block, error_ident, catch_block))
    }

    pub fn parse_return(&mut self) -> Result<Option<Stmt>> {
        debug!("Parsing return statement");
        let return_token = self.consume();
        let value = if self.peek().kind != TokenKind::Semicolon {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        self.possible_check(TokenKind::Semicolon);

        Ok(Some(Stmt::new_return(return_token, value)))
    }

    pub fn parse_let(&mut self) -> Result<Stmt> {
        debug!("Parsing let statement");
        self.expect(TokenKind::Let)?;
        let ident = self.expect(TokenKind::Identifier)?;
        let type_annotation = self.parse_optional_type_annotation()?;
        self.expect(TokenKind::Equals)?;
        let value = self.parse_expr()?;
        Ok(Stmt::new_let(ident, Box::new(value), type_annotation))
    }

    pub fn parse_if(&mut self) -> Result<Stmt> {
        debug!("Parsing if statement");
        let if_token = self.consume();

        self.possible_check(TokenKind::LeftParen);

        let condition = self.parse_expr()?;

        self.possible_check(TokenKind::RightParen);

        self.expect(TokenKind::LeftBrace)?;

        let body = self.parse_block()?;

        self.expect(TokenKind::RightBrace)?;

        let mut elseif_blocks = vec![];
        let mut else_block: Option<ElseBlock> = None;

        while self.peek().kind == TokenKind::Else {
            self.consume();

            if self.peek().kind == TokenKind::If {
                self.consume();
                self.possible_check(TokenKind::LeftParen);

                let condition = self.parse_expr()?;
                self.possible_check(TokenKind::RightParen);

                self.expect(TokenKind::LeftBrace)?;
                let body = self.parse_block()?;
                self.expect(TokenKind::RightBrace)?;

                elseif_blocks.push(ElseBlock {
                    condition: Box::new(condition),
                    block: body,
                    else_if: true,
                });
            } else {
                self.expect(TokenKind::LeftBrace)?;

                let body = self.parse_block()?;

                self.expect(TokenKind::RightBrace)?;

                else_block = Some(ElseBlock {
                    condition: Box::new(condition.clone()),
                    block: body,
                    else_if: false,
                });
            }
        }

        Ok(Stmt::new_if(
            if_token,
            condition.into(),
            body,
            elseif_blocks.into(),
            else_block,
        ))
    }

    pub fn parse_use(&mut self) -> Result<Stmt> {
        debug!("Parsing use statement");
        let use_token = self.consume();

        let mut items = vec![];

        self.expect(TokenKind::LeftBrace)?;

        while self.peek().kind != TokenKind::RightBrace && !self.is_eof() {
            let item = self.expect(TokenKind::Identifier)?;

            if self.peek().kind != TokenKind::RightBrace {
                self.expect(TokenKind::Comma)?;
            }

            items.push(item);
        }

        self.expect(TokenKind::RightBrace)?;

        self.expect(TokenKind::From)?;

        let from = if self.peek().is_string() {
            self.consume()
        } else {
            return Err(ExpectedToken(
                "string".to_string(),
                "Expected string that is valid module or file".to_string(),
                self.peek().span.clone(),
            )
                .into());
        };

        Ok(Stmt::new_use(use_token, from, items))
    }

    pub fn parse_type_annotation(&mut self) -> Result<TypeAnnotation> {
        debug!("Parsing type annotation");
        let colon = self.expect(TokenKind::Colon)?;
        let type_name = self.expect(TokenKind::Identifier)?;

        Ok(TypeAnnotation { colon, type_name })
    }

    pub fn parse_return_type(&mut self) -> Result<Option<FunctionType>> {
        debug!("Parsing return type");
        if self.peek().kind == TokenKind::Identifier {
            Err(ExpectedToken(
                "arrow".to_string(),
                "Expected arrow".to_string(),
                self.peek().span.clone(),
            )
                .into())
        } else {
            let arrow = self.consume();
            let type_name = self.expect(TokenKind::Identifier)?;

            Ok(Some(FunctionType { arrow, type_name }))
        }
    }

    pub fn parse_block(&mut self) -> Result<Block> {
        debug!("Parsing block");
        let mut stmts = vec![];

        while self.peek().kind != TokenKind::RightBrace && !self.is_eof() {
            let stmt = self.parse_stmt()?;

            if let Some(stmt) = stmt {
                debug!("Adding statement to block");
                stmts.push(stmt);
            }
        }

        Ok(Block { stmts })
    }

    pub fn parse_fn(&mut self) -> Result<Stmt> {
        debug!("Parsing function");
        let mut exported = false;
        let fn_token = if self.peek().kind == TokenKind::Export {
            self.consume();

            if self.peek().kind == TokenKind::Fn {
                exported = true;

                self.consume()
            } else {
                return Err(ExpectedToken(
                    "function".to_string(),
                    "You can only export functions".to_string(),
                    self.peek().span.clone(),
                )
                    .into());
            }
        } else {
            self.consume()
        };
        let name = self.expect(TokenKind::Identifier)?;

        self.expect(TokenKind::LeftParen)?;
        let mut params = vec![];

        let mut has_rest_param = false;

        if self.peek().kind != TokenKind::RightParen {
            while self.peek().kind != TokenKind::RightParen && !self.is_eof() {
                self.possible_check(TokenKind::Comma);

                let is_rest = self.peek().kind == TokenKind::TripleDot;

                if is_rest {
                    if has_rest_param {
                        return Err(MultipleRestParameters(self.peek().span.clone()).into());
                    }
                    has_rest_param = true;
                    self.consume();
                }

                let param = self.consume();
                let type_annotation = self.parse_type_annotation()?;

                if has_rest_param && self.peek().kind != TokenKind::RightParen {
                    return Err(RestParameterNotLastPosition(param.span.clone()).into());
                }

                params.push(FnParam {
                    type_annotation,
                    ident: param,
                    is_rest,
                });
            }
        }

        self.expect(TokenKind::RightParen)?;

        let return_type = self.parse_return_type()?;

        self.expect(TokenKind::LeftBrace)?;

        let body = self.parse_block()?;

        self.expect(TokenKind::RightBrace)?;

        Ok(Stmt::new_fn(
            fn_token,
            name.literal(),
            params,
            body,
            exported,
            return_type,
        ))
    }
}

impl Parser {
    pub fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_assignment()
    }

    pub fn expression_stmt(&mut self) -> Result<Stmt> {
        let expr = self.parse_expr()?;

        self.possible_check(TokenKind::Semicolon);

        Ok(expr.into())
    }

    pub fn parse_binary_expression(&mut self) -> Result<Expr> {
        let left = self.parse_unary_expression()?;
        self.parse_binary_expression_recurse(left, 0)
    }

    fn parse_binary_operator(&mut self) -> Option<BinOperator> {
        let token = self.peek();
        let kind = match token.kind {
            TokenKind::Plus => Some(BinOpKind::Plus),
            TokenKind::Minus => Some(BinOpKind::Minus),
            TokenKind::Asterisk => Some(BinOpKind::Multiply),
            TokenKind::Slash => Some(BinOpKind::Divide),
            TokenKind::Ampersand => Some(BinOpKind::BitwiseAnd),
            TokenKind::Pipe => Some(BinOpKind::BitwiseOr),
            TokenKind::Caret => Some(BinOpKind::BitwiseXor),
            TokenKind::DoubleAsterisk => Some(BinOpKind::Power),
            TokenKind::EqualsEquals => Some(BinOpKind::Equals),
            TokenKind::BangEquals => Some(BinOpKind::NotEquals),
            TokenKind::LessThan => Some(BinOpKind::LessThan),
            TokenKind::LessThanEquals => Some(BinOpKind::LessThanOrEqual),
            TokenKind::GreaterThan => Some(BinOpKind::GreaterThan),
            TokenKind::GreaterThanEquals => Some(BinOpKind::GreaterThanOrEqual),
            TokenKind::Percent => Some(BinOpKind::Modulo),
            TokenKind::And => Some(BinOpKind::And),
            TokenKind::Or => Some(BinOpKind::Or),
            TokenKind::Increment => Some(BinOpKind::Increment),
            TokenKind::Decrement => Some(BinOpKind::Decrement),
            TokenKind::MinusEquals => Some(BinOpKind::MinusEquals),
            TokenKind::PlusEquals => Some(BinOpKind::PlusEquals),
            _ => None,
        };
        kind.map(|kind| BinOperator::new(kind, token.clone()))
    }

    pub fn parse_binary_expression_recurse(
        &mut self,
        mut left: Expr,
        precedence: u8,
    ) -> Result<Expr> {
        while let Some(operator) = self.parse_binary_operator() {
            let operator_precedence = operator.precedence();
            if operator_precedence < precedence {
                break;
            }

            self.consume();

            let mut right = self.parse_unary_expression()?;

            while let Some(next_operator) = self.parse_binary_operator() {
                let next_precedence = next_operator.precedence();

                if next_precedence > operator_precedence ||
                    (next_precedence == operator_precedence && next_operator.associativity() == BinOpAssociativity::Right) {
                    right = self.parse_binary_expression_recurse(right, next_precedence)?;
                } else {
                    break;
                }
            }

            left = Expr::new_binary(left, operator, right);
        }

        Ok(left)
    }


    pub fn parse_unary_operator(&mut self) -> Option<UnOperator> {
        let token = self.peek();
        let kind = match token.kind {
            TokenKind::Minus => Some(UnOpKind::Minus),
            TokenKind::Tilde => Some(UnOpKind::BitwiseNot),
            _ => None,
        };
        kind.map(|kind| UnOperator::new(kind, token.clone()))
    }

    pub fn parse_unary_expression(&mut self) -> Result<Expr> {
        if let Some(operator) = self.parse_unary_operator() {
            let token = self.consume();
            let operand = self.parse_unary_expression();
            return Ok(Expr::new_unary(operator, operand?, token));
        }
        self.parse_primary_expression()
    }

    pub fn parse_primary_expression(&mut self) -> Result<Expr> {
        let token = self.consume();

        match &token.kind.clone() {
            TokenKind::Integer(int) => Ok(Expr::new_integer(token, *int)),
            TokenKind::Float(float) => Ok(Expr::new_float(token, *float)),
            TokenKind::True | TokenKind::False => {
                Ok(Expr::new_bool(token.clone(), token.as_bool().unwrap()))
            }
            TokenKind::LeftBracket => self.parse_vector(),
            TokenKind::Identifier => {
                log::debug!("Parsing identifier: {}", token.literal());
                if self.peek().kind == TokenKind::LeftParen {
                    self.parse_call_expr(token)
                } else {
                    Ok(Expr::new_variable(token.clone(), token.literal()))
                }
            }
            TokenKind::LeftParen => {
                let expr = self.parse_expr()?;

                self.expect(TokenKind::RightParen)?;

                Ok(Expr::new_parenthesized(expr))
            }
            TokenKind::String(s) => Ok(Expr::new_string(token.clone(), s.clone())),
            _ => Err(UnexpectedToken(token.kind.to_string(), token.span.clone()).into()),
        }
    }

    pub fn parse_call_expr(&mut self, callee: Token) -> Result<Expr> {
        self.expect(TokenKind::LeftParen)?;

        let mut args = vec![];

        if self.peek().kind != TokenKind::RightParen {
            while self.peek().kind != TokenKind::RightParen && !self.is_eof() {
                let arg = self.parse_expr()?;

                args.push(arg);

                if self.peek().kind != TokenKind::RightParen {
                    self.expect(TokenKind::Comma)?;
                }
            }
        }

        self.expect(TokenKind::RightParen)?;

        Ok(Expr::new_call(callee.literal(), args, callee))
    }

    pub fn parse_optional_type_annotation(&mut self) -> Result<Option<TypeAnnotation>> {
        if self.peek().kind == TokenKind::Colon {
            Ok(Some(self.parse_type_annotation()?))
        } else {
            Ok(None)
        }
    }

    pub fn parse_vector(&mut self) -> Result<Expr> {
        debug!("Parsing vector");

        let mut elements = vec![];
        if self.peek().kind != TokenKind::RightBracket {
            while self.peek().kind != TokenKind::RightBracket && !self.is_eof() {
                let arg = self.parse_expr()?;

                elements.push(arg);

                if self.peek().kind != TokenKind::RightBracket {
                    self.expect(TokenKind::Comma)?;
                }
            }
        }

        self.expect(TokenKind::RightBracket)?;

        Ok(Expr::new_vec(elements))
    }

    pub fn parse_assignment(&mut self) -> Result<Expr> {
        log::debug!("Parsing assignment");
        if self.peek().kind == TokenKind::Identifier && self.peek_next().kind == TokenKind::Equals {
            let ident = self.consume();
            let equals = self.consume();
            let value = self.parse_expr()?;

            Ok(Expr::new_assign(ident, equals, value))
        } else {
            Ok(self.parse_binary_expression()?)
        }
    }
}