use crate::{Block, ElseBlock, FnParam, FunctionType, Parser, Stmt, TokenKind, TypeAnnotation};
use log::debug;
use roan_error::error::PulseError::{
    ExpectedToken, MultipleRestParameters, RestParameterNotLastPosition,
};

impl Parser {
    /// Parses a statement from the tokens.
    ///
    /// This method checks the type of token to determine the kind of statement it should create.
    /// It supports function declarations, variable assignments, control flow, and more.
    ///
    /// # Returns
    /// - `Ok(Some(Stmt))`: A parsed statement.
    /// - `Ok(None)`: If the token is a comment or semicolon.
    /// - `Err`: If there is a parsing error.
    pub fn parse_stmt(&mut self) -> anyhow::Result<Option<Stmt>> {
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
            TokenKind::Semicolon | TokenKind::Comment => {
                self.consume();
                None
            }
            _ => Some(self.expression_stmt()?),
        };

        Ok(stmt)
    }

    /// Parses a `throw` statement.
    ///
    /// A `throw` statement is used to raise an exception.
    ///
    /// # Returns
    /// - `Ok(Stmt)`: A throw statement.
    /// - `Err`: If there is a parsing error.
    pub fn parse_throw(&mut self) -> anyhow::Result<Stmt> {
        debug!("Parsing throw statement");
        let throw_token = self.consume();
        let value = self.parse_expr()?;

        self.possible_check(TokenKind::Semicolon);

        Ok(Stmt::new_throw(throw_token, value))
    }

    /// Parses a `try` statement with a `catch` block.
    ///
    /// The `try` statement lets you catch exceptions and handle errors in a safe way.
    ///
    /// # Returns
    /// - `Ok(Stmt)`: A try-catch statement.
    /// - `Err`: If there is a parsing error.
    pub fn parse_try(&mut self) -> anyhow::Result<Stmt> {
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

        Ok(Stmt::new_try(
            try_token,
            try_block,
            error_ident,
            catch_block,
        ))
    }

    /// Parses a `return` statement.
    ///
    /// The `return` statement is used to return a value from a function.
    ///
    /// # Returns
    /// - `Ok(Some(Stmt))`: A return statement with or without a value.
    /// - `Err`: If there is a parsing error.
    pub fn parse_return(&mut self) -> anyhow::Result<Option<Stmt>> {
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

    /// Parses a `let` statement.
    ///
    /// A `let` statement declares a new variable with an optional type annotation.
    ///
    /// # Returns
    /// - `Ok(Stmt)`: A variable declaration statement.
    /// - `Err`: If there is a parsing error.
    pub fn parse_let(&mut self) -> anyhow::Result<Stmt> {
        debug!("Parsing let statement");
        self.expect(TokenKind::Let)?;
        let ident = self.expect(TokenKind::Identifier)?;
        let type_annotation = self.parse_optional_type_annotation()?;
        self.expect(TokenKind::Equals)?;
        let value = self.parse_expr()?;
        Ok(Stmt::new_let(ident, Box::new(value), type_annotation))
    }

    /// Parses an `if` statement with optional `else if` and `else` blocks.
    ///
    /// An `if` statement is used for conditional logic.
    ///
    /// # Returns
    /// - `Ok(Stmt)`: An if statement with possible elseif and else blocks.
    /// - `Err`: If there is a parsing error.
    pub fn parse_if(&mut self) -> anyhow::Result<Stmt> {
        debug!("Parsing if statement");
        let if_token = self.consume();

        let condition = self.parse_expr()?;

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

    /// Parses a `use` statement for importing modules.
    ///
    /// A `use` statement allows importing items from other modules or files.
    ///
    /// # Returns
    /// - `Ok(Stmt)`: A use statement.
    /// - `Err`: If there is a parsing error.
    pub fn parse_use(&mut self) -> anyhow::Result<Stmt> {
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

    /// Parses a type annotation following a variable or parameter.
    ///
    /// A type annotation specifies the type of a variable or function parameter.
    ///
    /// # Returns
    /// - `Ok(TypeAnnotation)`: A parsed type annotation.
    /// - `Err`: If there is a parsing error.
    pub fn parse_type_annotation(&mut self) -> anyhow::Result<TypeAnnotation> {
        debug!("Parsing type annotation");
        let colon = self.expect(TokenKind::Colon)?;
        let type_name = self.expect(TokenKind::Identifier)?;

        Ok(TypeAnnotation { colon, type_name })
    }

    /// Parses the return type of a function.
    ///
    /// The return type indicates the type of value a function returns.
    ///
    /// # Returns
    /// - `Ok(Some(FunctionType))`: If the return type is parsed.
    /// - `Ok(None)`: If no return type is provided.
    /// - `Err`: If the syntax is incorrect.
    pub fn parse_return_type(&mut self) -> anyhow::Result<Option<FunctionType>> {
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

    /// Parses a block of statements enclosed by curly braces `{}`.
    ///
    /// A block is a group of statements that are executed in sequence.
    ///
    /// # Returns
    /// - `Ok(Block)`: A parsed block of statements.
    /// - `Err`: If there is a parsing error.
    pub fn parse_block(&mut self) -> anyhow::Result<Block> {
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

    /// Parses a function declaration.
    ///
    /// A function declaration defines a new function, including its parameters, return type, and body.
    ///
    /// # Returns
    /// - `Ok(Stmt)`: A function declaration.
    /// - `Err`: If there is a parsing error.
    pub fn parse_fn(&mut self) -> anyhow::Result<Stmt> {
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
