use crate::{
    AssignOperator, BinOpAssociativity, BinOpKind, BinOperator, Expr, ParseContext, Parser, Stmt,
    Token, TokenKind, TypeAnnotation, UnOpKind, UnOperator,
};
use indexmap::IndexMap;
use roan_error::error::PulseError::{ExpectedToken, UnexpectedToken};
use tracing::debug;

impl Parser {
    /// Parses any expression, starting with an assignment.
    ///
    /// This method serves as the entry point for expression parsing.
    ///
    /// # Returns
    /// - `Ok(Expr)`: The parsed expression if successful.
    /// - `Err(anyhow::Error)`: An error if parsing fails.
    pub fn parse_expr(&mut self) -> anyhow::Result<Expr> {
        self.parse_assignment()
    }

    /// Parses an expression statement.
    ///
    /// This method parses an expression and checks for a semicolon to terminate the statement.
    ///
    /// # Returns
    /// - `Ok(Stmt)`: The expression statement if successful.
    /// - `Err(anyhow::Error)`: An error if parsing fails.
    pub fn expression_stmt(&mut self) -> anyhow::Result<Stmt> {
        let expr = self.parse_expr()?;

        self.possible_check(TokenKind::Semicolon);

        Ok(expr.into())
    }

    /// Parses a binary expression.
    ///
    /// This method first parses a unary expression and then handles the binary operators in the expression.
    ///
    /// # Returns
    /// - `Ok(Expr)`: The parsed binary expression if successful.
    /// - `Err(anyhow::Error)`: An error if parsing fails.
    pub fn parse_binary_expression(&mut self) -> anyhow::Result<Expr> {
        let left = self.parse_unary_expression()?;
        self.parse_binary_expression_recurse(left, 0)
    }

    /// Attempts to parse a binary operator.
    ///
    /// This method checks the next token to see if it's a binary operator and returns it if found.
    ///
    /// # Returns
    /// - `Some(BinOperator)`: The parsed binary operator if found.
    /// - `None`: If no binary operator is found.
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
            TokenKind::BangEquals => Some(BinOpKind::BangEquals),
            TokenKind::LessThan => Some(BinOpKind::LessThan),
            TokenKind::LessThanEquals => Some(BinOpKind::LessThanOrEqual),
            TokenKind::GreaterThan => Some(BinOpKind::GreaterThan),
            TokenKind::GreaterThanEquals => Some(BinOpKind::GreaterThanOrEqual),
            TokenKind::Percent => Some(BinOpKind::Modulo),
            TokenKind::And => Some(BinOpKind::And),
            TokenKind::Or => Some(BinOpKind::Or),
            TokenKind::Increment => Some(BinOpKind::Increment),
            TokenKind::Decrement => Some(BinOpKind::Decrement),
            TokenKind::DoubleGreaterThan => Some(BinOpKind::ShiftRight),
            TokenKind::DoubleLessThan => Some(BinOpKind::ShiftLeft),
            _ => None,
        };
        kind.map(|kind| BinOperator::new(kind, token.clone()))
    }

    /// Parses binary expressions recursively, handling operator precedence and associativity.
    ///
    /// This method continues to parse binary expressions until no further valid operators are found.
    ///
    /// # Parameters
    /// - `left`: The left-hand side expression.
    /// - `precedence`: The precedence of the operator being processed.
    ///
    /// # Returns
    /// - `Ok(Expr)`: The final parsed expression if successful.
    /// - `Err(anyhow::Error)`: An error if parsing fails.
    pub fn parse_binary_expression_recurse(
        &mut self,
        mut left: Expr,
        precedence: u8,
    ) -> anyhow::Result<Expr> {
        while let Some(operator) = self.parse_binary_operator() {
            let operator_precedence = operator.precedence();
            if operator_precedence < precedence {
                break;
            }

            self.consume();

            let mut right = self.parse_unary_expression()?;

            while let Some(next_operator) = self.parse_binary_operator() {
                let next_precedence = next_operator.precedence();

                if next_precedence > operator_precedence
                    || (next_precedence == operator_precedence
                        && next_operator.associativity() == BinOpAssociativity::Right)
                {
                    right = self.parse_binary_expression_recurse(right, next_precedence)?;
                } else {
                    break;
                }
            }

            left = Expr::new_binary(left, operator, right);
        }

        Ok(left)
    }

    /// Attempts to parse a unary operator.
    ///
    /// This method checks the next token to see if it's a unary operator and returns it if found.
    ///
    /// # Returns
    /// - `Some(UnOperator)`: The parsed unary operator if found.
    /// - `None`: If no unary operator is found.
    pub fn parse_unary_operator(&mut self) -> Option<UnOperator> {
        let token = self.peek();
        let kind = match token.kind {
            TokenKind::Minus => Some(UnOpKind::Minus),
            TokenKind::Tilde => Some(UnOpKind::BitwiseNot),
            TokenKind::Bang => Some(UnOpKind::LogicalNot),
            _ => None,
        };
        kind.map(|kind| UnOperator::new(kind, token.clone()))
    }

    /// Parses a unary expression, handling unary operators.
    ///
    /// This method checks for a unary operator and processes the operand accordingly.
    ///
    /// # Returns
    /// - `Ok(Expr)`: The parsed unary expression if successful.
    /// - `Err(anyhow::Error)`: An error if parsing fails.
    pub fn parse_unary_expression(&mut self) -> anyhow::Result<Expr> {
        if let Some(operator) = self.parse_unary_operator() {
            let token = self.consume();
            let operand = self.parse_unary_expression()?;
            return Ok(Expr::new_unary(operator, operand, token));
        }
        self.parse_access_expression()
    }

    /// Parses an access expression.
    pub fn parse_access_expression(&mut self) -> anyhow::Result<Expr> {
        debug!("Parsing access expression");
        let mut expr = self.parse_primary_expression()?;
        let mut token = self.peek();

        loop {
            if token.kind == TokenKind::Dot {
                self.consume();

                let field_token = self.consume();
                let mut field_expr = Expr::new_variable(field_token.clone(), field_token.literal());

                if self.peek().kind == TokenKind::LeftParen {
                    field_expr = self.parse_call_expr(field_token)?;
                }

                expr = Expr::new_field_access(expr, field_expr, token);
            } else if token.kind == TokenKind::LeftBracket {
                self.consume();
                let index = self.parse_expr()?;
                self.expect(TokenKind::RightBracket)?;
                expr = Expr::new_index_access(expr, index, token);
            } else if token.kind == TokenKind::DoubleColon {
                let colons = self.consume();
                let field = self.parse_expr()?;

                expr = Expr::new_static_method_access(expr, field, colons);
            } else {
                break;
            }
            token = self.peek();
        }

        Ok(expr)
    }

    /// Parses a struct constructor expression.
    ///
    /// This method expects an identifier followed by a left brace and a list of field assignments.
    ///
    /// # Parameters
    /// - `identifier`: The token representing the struct name.
    ///
    /// # Returns
    /// - `Ok(Expr)`: The parsed struct constructor expression if successful.
    /// - `Err(anyhow::Error)`: An error if parsing fails.
    pub fn parse_struct_constructor(&mut self, identifier: Token) -> anyhow::Result<Expr> {
        self.expect(TokenKind::LeftBrace)?;

        let mut fields = vec![];

        while self.peek().kind != TokenKind::RightBrace && !self.is_eof() {
            let field_name = self.consume();
            self.expect(TokenKind::Colon)?;
            let field_value = self.parse_expr()?;

            fields.push((field_name.literal(), field_value));

            if self.peek().kind != TokenKind::RightBrace {
                self.expect(TokenKind::Comma)?;
            }
        }

        self.expect(TokenKind::RightBrace)?;

        Ok(Expr::new_struct_constructor(
            identifier.literal(),
            fields,
            identifier,
        ))
    }

    /// Parses a primary expression, such as literals, identifiers, or parenthesized expressions.
    ///
    /// # Returns
    /// - `Ok(Expr)`: The parsed primary expression if successful.
    /// - `Err(anyhow::Error)`: An error if parsing fails.
    pub fn parse_primary_expression(&mut self) -> anyhow::Result<Expr> {
        let token = self.consume();

        match &token.kind {
            TokenKind::Integer(int) => Ok(Expr::new_integer(token.clone(), *int)),
            TokenKind::Float(float) => Ok(Expr::new_float(token.clone(), *float)),
            TokenKind::Null => Ok(Expr::new_null(token)),
            TokenKind::True | TokenKind::False => {
                Ok(Expr::new_bool(token.clone(), token.as_bool().unwrap()))
            }
            TokenKind::TripleDot => Ok(Expr::new_spread(token.clone(), self.parse_expr()?)),
            TokenKind::LeftBracket => self.parse_vector(),
            TokenKind::LeftBrace => {
                let mut fields: IndexMap<String, Expr> = IndexMap::new();

                while self.peek().kind != TokenKind::RightBrace && !self.is_eof() {
                    let field_name = {
                        if matches!(self.peek().kind, TokenKind::String(_)) {
                            self.consume()
                        } else {
                            return Err(ExpectedToken(
                                "string literal".to_string(),
                                "Field names in objects must be string literals.".to_string(),
                                self.peek().span.clone(),
                            )
                            .into());
                        }
                    };

                    self.expect(TokenKind::Colon)?;
                    let field_value = self.parse_expr()?;

                    fields.insert(
                        field_name
                            .literal()
                            .strip_prefix("\"")
                            .unwrap()
                            .strip_suffix("\"")
                            .unwrap()
                            .to_string(),
                        field_value,
                    );

                    if self.peek().kind != TokenKind::RightBrace {
                        self.expect(TokenKind::Comma)?;
                    }
                }

                let closing_brace = self.expect(TokenKind::RightBrace)?;

                Ok(Expr::new_object(fields, (token, closing_brace)))
            }
            TokenKind::Identifier => {
                debug!("Parsing identifier: {}", token.literal());

                if self.peek().kind == TokenKind::LeftParen {
                    self.parse_call_expr(token)
                } else if self.peek().kind == TokenKind::LeftBrace {
                    if self.is_context(&ParseContext::Normal) {
                        self.parse_struct_constructor(token)
                    } else {
                        Ok(Expr::new_variable(token.clone(), token.literal()))
                    }
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
            TokenKind::Char(c) => Ok(Expr::new_char(token.clone(), *c)),
            _ => {
                debug!("Unexpected token: {:?}", token);
                Err(UnexpectedToken(token.kind.to_string(), token.span.clone()).into())
            }
        }
    }

    /// Parses a then-else expression.
    ///
    /// This method expects an identifier followed by a then keyword and two expressions.
    ///
    /// # Parameters
    /// - `identifier`: The token representing the identifier.
    ///
    /// # Returns
    /// - `Ok(Expr)`: The parsed then-else expression if successful.
    /// - `Err(anyhow::Error)`: An error if parsing fails.
    pub fn parse_then_else_expr(&mut self, condition: Expr) -> anyhow::Result<Expr> {
        debug!("Parsing then-else expression");
        let then_token = self.expect(TokenKind::Then)?;

        let then_expr = self.parse_expr()?;
        let else_token = self.expect(TokenKind::Else)?;

        let else_expr = self.parse_expr()?;

        Ok(Expr::new_then_else(
            condition, then_expr, else_expr, then_token, else_token,
        ))
    }

    /// Parses a function call expression.
    ///
    /// This method expects an identifier followed by parentheses containing arguments.
    ///
    /// # Parameters
    /// - `callee`: The token representing the function name.
    ///
    /// # Returns
    /// - `Ok(Expr)`: The parsed call expression if successful.
    /// - `Err(anyhow::Error)`: An error if parsing fails.
    pub fn parse_call_expr(&mut self, callee: Token) -> anyhow::Result<Expr> {
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

    /// Parses an optional type annotation.
    ///
    /// This method checks for a colon followed by a type annotation and parses it if present.
    ///
    /// # Returns
    /// - `Ok(Some(TypeAnnotation))`: The parsed type annotation if present.
    /// - `Ok(None)`: If no type annotation is present.
    pub fn parse_optional_type_annotation(&mut self) -> anyhow::Result<Option<TypeAnnotation>> {
        if self.peek().kind == TokenKind::Colon {
            Ok(Some(self.parse_type_annotation()?))
        } else {
            Ok(None)
        }
    }

    /// Parses a vector expression.
    ///
    /// This method expects a left bracket followed by a list of expressions and a closing right bracket.
    ///
    /// # Returns
    /// - `Ok(Expr)`: The parsed vector expression if successful.
    /// - `Err(anyhow::Error)`: An error if parsing fails.
    pub fn parse_vector(&mut self) -> anyhow::Result<Expr> {
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

    /// Parses an assignment expression.
    ///
    /// This method checks for an identifier followed by an equals sign and an expression.
    ///
    /// # Returns
    /// - `Ok(Expr)`: The parsed assignment expression if successful.
    /// - `Err(anyhow::Error)`: An error if parsing fails.
    pub fn parse_assignment(&mut self) -> anyhow::Result<Expr> {
        tracing::debug!("Parsing assignment");

        let expr = self.parse_binary_expression()?;
        if let Some(assign_op) = self.parse_assignment_operator() {
            self.consume();
            let right = self.parse_expr()?;

            let operator = AssignOperator::from_token_kind(assign_op);
            return Ok(Expr::new_assign(expr, operator, right));
        } else if self.peek().kind == TokenKind::Then {
            return self.parse_then_else_expr(expr);
        }

        Ok(expr)
    }

    /// Attempts to parse an assignment operator.
    ///
    /// This method checks the next token to see if it's an assignment operator and returns it if found.
    ///
    /// # Returns
    /// - `Some(TokenKind)`: The parsed assignment operator if found.
    /// - `None`: If no assignment operator is found.
    fn parse_assignment_operator(&mut self) -> Option<TokenKind> {
        match self.peek().kind {
            TokenKind::Equals
            | TokenKind::PlusEquals
            | TokenKind::MinusEquals
            | TokenKind::MultiplyEquals
            | TokenKind::DivideEquals => Some(self.peek().kind.clone()),
            _ => None,
        }
    }
}
