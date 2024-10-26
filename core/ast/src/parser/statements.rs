use crate::{
    Block, ElseBlock, FnParam, FunctionType, ParseContext, Parser, Stmt, StructField, Token,
    TokenKind, TypeAnnotation,
};
use anyhow::Result;
use colored::Colorize;
use roan_error::error::PulseError::{
    ExpectedToken, InvalidType, MultipleRestParameters, MultipleSelfParameters,
    RestParameterNotLastPosition, SelfParameterCannotBeRest, SelfParameterNotFirst,
};
use tracing::debug;

static VALID_TYPE_NAMES: [&str; 7] = ["bool", "int", "float", "string", "void", "anytype", "char"];

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
    pub fn parse_stmt(&mut self) -> Result<Option<Stmt>> {
        let token = self.peek();

        let stmt = match token.kind {
            TokenKind::Pub => {
                if self.peek_next().kind == TokenKind::Fn {
                    Some(self.parse_fn()?)
                } else if self.peek_next().kind == TokenKind::Struct {
                    Some(self.parse_struct()?)
                } else if self.peek_next().kind == TokenKind::Trait {
                    Some(self.parse_trait()?)
                } else if self.peek_next().kind == TokenKind::Const {
                    Some(self.parse_const()?)
                } else {
                    // TODO: return error
                    None
                }
            }
            TokenKind::Fn => Some(self.parse_fn()?),
            TokenKind::Struct => Some(self.parse_struct()?),
            TokenKind::Trait => Some(self.parse_trait()?),
            TokenKind::Const => Some(self.parse_const()?),
            TokenKind::Impl => {
                let impl_keyword = self.consume();
                if self.peek().kind == TokenKind::Identifier {
                    let ident = self.consume();

                    if self.peek().kind == TokenKind::For {
                        Some(self.parse_trait_impl(impl_keyword, ident)?)
                    } else if self.peek().kind == TokenKind::LeftBrace {
                        Some(self.parse_impl(impl_keyword, ident)?)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            TokenKind::Use => Some(self.parse_use()?),
            TokenKind::If => Some(self.parse_if()?),
            TokenKind::Let => Some(self.parse_let()?),
            TokenKind::Throw => Some(self.parse_throw()?),
            TokenKind::Try => Some(self.parse_try()?),
            TokenKind::Break => {
                self.consume();
                self.possible_check(TokenKind::Semicolon);
                Some(Stmt::new_break(token))
            }
            TokenKind::Continue => {
                self.consume();
                self.possible_check(TokenKind::Semicolon);
                Some(Stmt::new_continue(token))
            }
            TokenKind::Loop => {
                self.consume();
                let block = self.parse_block()?;
                Some(Stmt::new_loop(token, block))
            }
            TokenKind::While => self.parse_while()?,
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

    /// Parses an `impl` block for implementing a struct.
    ///
    /// An `impl` block is used to implement methods for a struct.
    ///
    /// # Returns
    /// - `Ok(Stmt)`: An impl block.
    /// - `Err`: If there is a parsing error.
    pub fn parse_impl(&mut self, impl_keyword: Token, ident: Token) -> Result<Stmt> {
        debug!("Parsing impl block");
        self.expect(TokenKind::LeftBrace)?;

        let mut methods: Vec<crate::Fn> = vec![];

        while self.peek().kind != TokenKind::RightBrace && !self.is_eof() {
            let func = self.parse_fn()?.into_function();

            methods.push(func);
        }

        self.expect(TokenKind::RightBrace)?;

        Ok(Stmt::new_struct_impl(impl_keyword, ident, methods))
    }

    /// Parses an `impl` block for implementing a trait.
    ///
    /// An `impl` block is used to implement methods for a trait.
    ///
    /// # Returns
    /// - `Ok(Stmt)`: An impl block.
    /// - `Err`: If there is a parsing error.
    pub fn parse_trait_impl(&mut self, impl_keyword: Token, ident: Token) -> Result<Stmt> {
        debug!("Parsing impl block");
        let for_token = self.expect(TokenKind::For)?;

        let trait_name = self.expect(TokenKind::Identifier)?;

        self.expect(TokenKind::LeftBrace)?;

        let mut methods: Vec<crate::Fn> = vec![];

        while self.peek().kind != TokenKind::RightBrace && !self.is_eof() {
            let func = self.parse_fn()?.into_function();

            methods.push(func);
        }

        self.expect(TokenKind::RightBrace)?;

        Ok(Stmt::new_trait_impl(
            impl_keyword,
            ident,
            for_token,
            trait_name,
            methods,
        ))
    }

    /// Parses a 'pub' keyword (if present) followed by an identifier.
    pub fn parse_pub(&mut self, expected: TokenKind) -> Result<(Token, bool)> {
        let mut public = false;
        let token = if self.peek().kind == TokenKind::Pub {
            self.consume();
            public = true;
            self.consume()
        } else {
            self.consume()
        };

        Ok((token, public))
    }

    /// Parses a `trait` declaration.
    ///
    /// A `trait` declaration defines a new interface that can be implemented by other types.
    ///
    /// # Returns
    /// - `Ok(Stmt)`: A trait declaration.
    /// - `Err`: If there is a parsing error.
    pub fn parse_trait(&mut self) -> Result<Stmt> {
        debug!("Parsing trait");
        let (trait_token, public) = self.parse_pub(TokenKind::Trait)?;

        let name = self.expect(TokenKind::Identifier)?;

        self.expect(TokenKind::LeftBrace)?;

        let mut methods: Vec<crate::Fn> = vec![];

        while self.peek().kind != TokenKind::RightBrace && !self.is_eof() {
            let func = self.parse_fn()?.into_function();
            methods.push(func);
        }

        self.expect(TokenKind::RightBrace)?;

        Ok(Stmt::new_trait_def(trait_token, name, methods, public))
    }

    /// Parses an expression statement.
    ///
    /// An expression statement is a statement that consists of an expression followed by a semicolon.
    ///
    /// # Returns
    ///
    /// - `Stmt`: An expression statement.
    /// - `Err`: If there is a parsing error.
    pub fn parse_const(&mut self) -> Result<Stmt> {
        debug!("Parsing const");
        let (_, public) = self.parse_pub(TokenKind::Const)?;

        let name = self.expect(TokenKind::Identifier)?;

        self.expect(TokenKind::Equals)?;

        let expr = self.parse_expr()?;

        Ok(Stmt::new_const(Box::new(expr), name, public))
    }

    /// Parses a `struct` declaration.
    ///
    /// A `struct` declaration defines a new data structure with named fields.
    ///
    /// # Returns
    /// - `Ok(Stmt)`: A struct declaration.
    /// - `Err`: If there is a parsing error.
    pub fn parse_struct(&mut self) -> Result<Stmt> {
        debug!("Parsing struct");
        let (struct_token, public) = self.parse_pub(TokenKind::Struct)?;
        let name = self.expect(TokenKind::Identifier)?;

        self.expect(TokenKind::LeftBrace)?;

        let mut fields: Vec<StructField> = vec![];

        while self.peek().kind != TokenKind::RightBrace && !self.is_eof() {
            let ident = self.expect(TokenKind::Identifier)?;
            let type_annotation = self.parse_type_annotation()?;
            fields.push(StructField {
                ident,
                type_annotation,
            });

            if self.peek().kind != TokenKind::RightBrace {
                self.expect(TokenKind::Comma)?;
            }
        }

        self.expect(TokenKind::RightBrace)?;

        Ok(Stmt::new_struct(struct_token, name, fields, public))
    }

    /// Parses a `while` statement.
    ///
    /// A `while` statement is used to execute a block of code repeatedly as long as a condition is true.
    ///
    /// # Returns
    /// - `Ok(Stmt)`: A while statement.
    /// - `Err`: If there is a parsing error.
    pub fn parse_while(&mut self) -> Result<Option<Stmt>> {
        debug!("Parsing while statement");
        let while_token = self.consume();

        self.push_context(ParseContext::WhileCondition);
        let condition = self.parse_expr()?;
        self.pop_context();

        self.expect(TokenKind::LeftBrace)?;
        let block = self.parse_block()?;
        self.expect(TokenKind::RightBrace)?;

        Ok(Some(Stmt::new_while(while_token, condition, block)))
    }

    /// Parses a `throw` statement.
    ///
    /// A `throw` statement is used to raise an exception.
    ///
    /// # Returns
    /// - `Ok(Stmt)`: A throw statement.
    /// - `Err`: If there is a parsing error.
    pub fn parse_throw(&mut self) -> Result<Stmt> {
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

    /// Parses a `let` statement.
    ///
    /// A `let` statement declares a new variable with an optional type annotation.
    ///
    /// # Returns
    /// - `Ok(Stmt)`: A variable declaration statement.
    /// - `Err`: If there is a parsing error.
    pub fn parse_let(&mut self) -> Result<Stmt> {
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
    pub fn parse_if(&mut self) -> Result<Stmt> {
        debug!("Parsing if statement");
        let if_token = self.consume();

        self.push_context(ParseContext::IfCondition);

        let condition = self.parse_expr()?;

        self.pop_context();

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

                self.push_context(ParseContext::IfCondition);
                let elseif_condition = self.parse_expr()?;
                self.pop_context();

                self.possible_check(TokenKind::RightParen);

                self.expect(TokenKind::LeftBrace)?;
                let elseif_body = self.parse_block()?;
                self.expect(TokenKind::RightBrace)?;

                elseif_blocks.push(ElseBlock {
                    condition: Box::new(elseif_condition),
                    block: elseif_body,
                    else_if: true,
                });
            } else {
                self.expect(TokenKind::LeftBrace)?;
                let else_body = self.parse_block()?;
                self.expect(TokenKind::RightBrace)?;

                else_block = Some(ElseBlock {
                    condition: Box::new(condition.clone()),
                    block: else_body,
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

    /// Checks if the next token is a question mark and consumes it.
    pub fn is_nullable(&mut self) -> bool {
        if self.peek().kind == TokenKind::QuestionMark {
            self.consume();
            true
        } else {
            false
        }
    }

    /// Helper method to parse a type with optional array and nullability.
    fn parse_type(&mut self) -> Result<(Token, bool)> {
        let type_name = self.expect(TokenKind::Identifier)?;
        Parser::validate_type_name(type_name.clone())?;

        let is_array = if self.peek().kind == TokenKind::LeftBracket {
            self.consume();
            self.expect(TokenKind::RightBracket)?;
            true
        } else {
            false
        };

        Ok((type_name, is_array))
    }

    /// Parses a type annotation following a variable or parameter.
    ///
    /// # Returns
    /// - `Ok(TypeAnnotation)`: A parsed type annotation.
    /// - `Err`: If there is a parsing error.
    pub fn parse_type_annotation(&mut self) -> Result<TypeAnnotation> {
        debug!("Parsing type annotation");
        let colon = self.expect(TokenKind::Colon)?;
        let (type_name, is_array) = self.parse_type()?;

        Ok(TypeAnnotation {
            type_name,
            is_array,
            is_nullable: self.is_nullable(),
            colon,
        })
    }

    /// Parses the return type of function.
    ///
    /// # Returns
    /// - `Ok(Some(FunctionType))`: If the return type is parsed.
    /// - `Ok(None)`: If no return type is provided.
    /// - `Err`: If the syntax is incorrect.
    pub fn parse_return_type(&mut self) -> Result<Option<FunctionType>> {
        debug!("Parsing return type");

        if self.peek().kind != TokenKind::Arrow {
            return Ok(None);
        }

        let arrow = self.consume(); // consume the arrow
        let (type_name, is_array) = self.parse_type()?;

        Ok(Some(FunctionType {
            type_name,
            is_array,
            is_nullable: self.is_nullable(),
            arrow,
        }))
    }

    /// Validates if the provided string is valid type name.
    ///
    /// # Returns
    /// - `Ok(())`: If the type name is valid.
    /// - `Err`: If the type name is invalid.
    pub fn validate_type_name(token: Token) -> Result<()> {
        let name = token.literal();

        debug!("Validating type name: {}", name);

        if !VALID_TYPE_NAMES.contains(&&*name) {
            debug!("Invalid type name: {}", name);
            return Err(InvalidType(
                name.cyan().to_string(),
                VALID_TYPE_NAMES.join(", "),
                token.span.clone(),
            )
            .into());
        }

        Ok(())
    }

    /// Parses a block of statements enclosed by curly braces `{}`.
    ///
    /// A block is a group of statements that are executed in sequence.
    ///
    /// # Returns
    /// - `Ok(Block)`: A parsed block of statements.
    /// - `Err`: If there is a parsing error.
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

    /// Parses a function declaration.
    ///
    /// A function declaration defines a new function, including its parameters, return type, and body.
    ///
    /// # Returns
    /// - `Ok(Stmt)`: A function declaration.
    /// - `Err`: If there is a parsing error.
    pub fn parse_fn(&mut self) -> Result<Stmt> {
        debug!("Parsing function");
        self.possible_check(TokenKind::Comment);

        let (fn_token, public) = self.parse_pub(TokenKind::Fn)?;

        let name = self.expect(TokenKind::Identifier)?;

        self.expect(TokenKind::LeftParen)?;
        let mut params = vec![];

        let mut has_rest_param = false;
        let mut is_static = true;

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

                if param.literal() == "self" {
                    if !is_static {
                        return Err(MultipleSelfParameters(self.peek().span.clone()).into());
                    }

                    is_static = false;

                    if is_rest {
                        return Err(SelfParameterCannotBeRest(self.peek().span.clone()).into());
                    }
                }

                let type_annotation = self.parse_optional_type_annotation()?;

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

        if !is_static && params[0].ident.literal() != "self" {
            return Err(SelfParameterNotFirst(self.peek().span.clone()).into());
        }

        self.expect(TokenKind::RightParen)?;

        let return_type = self.parse_return_type()?;

        let mut body = Block { stmts: vec![] };
        if self.peek().kind != TokenKind::LeftBrace {
            self.expect(TokenKind::Semicolon)?;
        } else {
            self.expect(TokenKind::LeftBrace)?;
            body = self.parse_block()?;
            self.expect(TokenKind::RightBrace)?;
        }

        Ok(Stmt::new_fn(
            fn_token,
            name.literal(),
            params,
            body,
            public,
            return_type,
            is_static,
        ))
    }
}
