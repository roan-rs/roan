use log::debug;
use roan_ast::{AccessKind, Assign, AssignOperator, BinOpKind, Binary, CallExpr, Expr, GetSpan, LiteralType, Spread};
use roan_error::error::PulseError;
use roan_error::error::PulseError::{InvalidSpread, PropertyNotFoundError, UndefinedFunctionError, VariableNotFoundError};
use crate::context::Context;
use crate::module::{Module, StoredFunction};
use crate::value::Value;
use anyhow::Result;
use roan_error::print_diagnostic;

impl Module {
    /// Interpret an expression.
    ///
    /// Result of the expression is pushed onto the stack.
    ///
    /// # Arguments
    /// * `expr` - [Expr] to interpret.
    /// * `ctx` - The context in which to interpret the expression.
    ///
    /// # Returns
    /// The result of the expression.
    pub fn interpret_expr(&mut self, expr: &Expr, ctx: &Context) -> Result<()> {
        let val: Result<Value> = match expr {
            Expr::Variable(v) => {
                debug!("Interpreting variable: {}", v.ident);

                let variable = self
                    .find_variable(&v.ident)
                    .ok_or_else(|| VariableNotFoundError(v.ident.clone(), v.token.span.clone()))?;

                Ok(variable.clone())
            }
            Expr::Literal(l) => {
                debug!("Interpreting literal: {:?}", l);

                Ok(Value::from_literal(l.clone()))
            }
            Expr::Call(call) => self.interpret_call(call, ctx),
            Expr::Parenthesized(p) => {
                debug!("Interpreting parenthesized: {:?}", p);

                self.interpret_expr(&p.expr, ctx)?;

                Ok(self.vm.pop().unwrap())
            }
            Expr::Access(access) => match access.access.clone() {
                AccessKind::Field(field_expr) => {
                    let base = access.base.clone();

                    self.interpret_expr(&base, ctx)?;
                    let base = self.vm.pop().unwrap();

                    Ok(self.access_field(base, &field_expr, ctx)?)
                }
                AccessKind::Index(index_expr) => {
                    self.interpret_expr(&index_expr, ctx)?;
                    let index = self.vm.pop().unwrap();

                    self.interpret_expr(&access.base, ctx)?;
                    let base = self.vm.pop().unwrap();

                    Ok(base.access_index(index))
                }
            },
            Expr::Assign(assign) => self.interpret_assignment(assign.clone(), ctx),
            Expr::Vec(vec) => {
                debug!("Interpreting vec: {:?}", vec);

                let mut values = vec![];

                for expr in vec.exprs.iter() {
                    self.interpret_expr(expr, ctx)?;
                    values.push(self.vm.pop().unwrap());
                }

                Ok(Value::Vec(values))
            }
            Expr::Binary(b) => self.interpret_binary(b.clone(), ctx),
            // Spread operator are only supposed to be used in vectors and function calls
            Expr::Spread(s) => Err(InvalidSpread(s.expr.span()).into()),

            _ => todo!("missing expr: {:?}", expr),
        };

        self.vm.push(val?);

        Ok(())
    }

    /// Interpret a call expression.
    ///
    /// # Arguments
    /// * `call` - [CallExpr] to interpret.
    /// * `ctx` - The context in which to interpret the call.
    ///
    /// # Returns
    /// The result of the call.
    pub fn interpret_call(&mut self, call: &CallExpr, ctx: &Context) -> Result<Value> {
        debug!("Interpreting call: {:?}", call);

        let mut args = vec![];
        for arg in call.args.iter() {
            self.interpret_expr(arg, ctx)?;
            args.push(self.vm.pop().expect("Expected value on stack"));
        }

        let stored_function = self
            .find_function(&call.callee)
            .ok_or_else(|| {
                UndefinedFunctionError(call.callee.clone(), call.token.span.clone())
            })?
            .clone();

        match stored_function {
            StoredFunction::Native(n) => {
                self.execute_native_function(n, args)?;
            }
            StoredFunction::Function {
                function,
                defining_module,
            } => {
                match self.execute_user_defined_function(function, defining_module.clone(), args, ctx) {
                    Ok(_) => {},
                    Err(e) => {
                        print_diagnostic(e, Some(defining_module.lock().unwrap().source.content()));
                        std::process::exit(1);
                    },
                }
            }
        }

        Ok(self.vm.pop().unwrap())
    }

    /// Interpret a binary expression.
    ///
    /// # Arguments
    /// * `binary_expr` - [Binary] expression to interpret.
    /// * `ctx` - The context in which to interpret the binary expression.
    ///
    /// # Returns
    /// The result of the binary expression.
    pub fn interpret_binary(&mut self, binary_expr: Binary, ctx: &Context) -> Result<Value> {
        debug!("Interpreting binary: {:?}", binary_expr);

        self.interpret_expr(&binary_expr.left, ctx)?;
        let left = self.vm.pop().unwrap();
        self.interpret_expr(&binary_expr.right, ctx)?;
        let right = self.vm.pop().unwrap();

        let val = match (left.clone(), binary_expr.operator, right.clone()) {
            (_, BinOpKind::Plus, _) => left + right,
            (_, BinOpKind::Minus, _) => left - right,
            (_, BinOpKind::Multiply, _) => left * right,
            (_, BinOpKind::Divide, _) => left / right,
            (_, BinOpKind::Modulo, _) => left % right,
            (_, BinOpKind::Equals, _) => Value::Bool(left == right),
            (_, BinOpKind::BangEquals, _) => Value::Bool(left != right),
            (_, BinOpKind::Power, _) => left.pow(right),

            (_, BinOpKind::GreaterThan, _) => Value::Bool(left > right),
            (_, BinOpKind::LessThan, _) => Value::Bool(left < right),
            (_, BinOpKind::GreaterThanOrEqual, _) => Value::Bool(left >= right),
            (_, BinOpKind::LessThanOrEqual, _) => Value::Bool(left <= right),

            (Value::Bool(a), BinOpKind::And, Value::Bool(b)) => Value::Bool(a && b),
            (Value::Bool(a), BinOpKind::Or, Value::Bool(b)) => Value::Bool(a || b),

            // TODO: add more bitwise operators
            (Value::Int(a), BinOpKind::BitwiseAnd, Value::Int(b)) => Value::Int(a & b),
            (Value::Int(a), BinOpKind::BitwiseOr, Value::Int(b)) => Value::Int(a | b),
            (Value::Int(a), BinOpKind::BitwiseXor, Value::Int(b)) => Value::Int(a ^ b),

            _ => todo!("missing binary operator: {:?}", binary_expr.operator),
        };

        Ok(val)
    }

    /// Interpret an assignment expression.
    ///
    /// # Arguments
    /// * `assign` - [Assign] expression to interpret.
    /// * `ctx` - The context in which to interpret the assignment expression.
    ///
    /// # Returns
    /// The result of the assignment expression.
    pub fn interpret_assignment(
        &mut self,
        assign: Assign,
        ctx: &Context,
    ) -> Result<Value> {
        debug!("Interpreting assign: {:?}", assign);
        let left = assign.left.as_ref();
        let right = assign.right.as_ref();
        let operator = assign.op.clone();

        debug!("{:?} \n\n{:?}\n\n {:?}", left, operator, right);

        match left {
            Expr::Variable(v) => {
                self.interpret_expr(right, ctx)?;
                let val = self.vm.pop().unwrap();
                let ident = v.ident.clone();
                let final_val = val.clone();
                match operator {
                    AssignOperator::Assign => self.set_variable(&ident, val.clone())?,
                    AssignOperator::PlusEquals => {
                        self.update_variable(&ident, val, |a, b| a + b)?
                    }
                    AssignOperator::MinusEquals => {
                        self.update_variable(&ident, val, |a, b| a - b)?
                    }
                    AssignOperator::MultiplyEquals => {
                        self.update_variable(&ident, val, |a, b| a * b)?
                    }
                    AssignOperator::DivideEquals => {
                        self.update_variable(&ident, val, |a, b| a / b)?
                    }
                }
                Ok(final_val)
            }
            Expr::Access(access) => match &access.access {
                AccessKind::Field(field) => {
                    let base = access.base.clone();

                    self.interpret_expr(right, ctx)?;
                    let new_val = self.vm.pop().unwrap();
                    unimplemented!("field access")
                }
                AccessKind::Index(index_expr) => {
                    self.interpret_expr(&access.base, ctx)?;
                    let base_val = self.vm.pop().unwrap();

                    self.interpret_expr(index_expr, ctx)?;
                    let index_val = self.vm.pop().unwrap();

                    self.interpret_expr(right, ctx)?;
                    let new_val = self.vm.pop().unwrap();

                    if let (Value::Vec(mut vec), Value::Int(index)) =
                        (base_val.clone(), index_val)
                    {
                        let idx = index as usize;
                        if idx >= vec.len() {
                            return Err(PulseError::IndexOutOfBounds(
                                idx,
                                vec.len(),
                                index_expr.span(),
                            )
                                .into());
                        }

                        vec[idx] = new_val.clone();

                        if let Some(var_name) = Self::extract_variable_name(&access.base) {
                            self.set_variable(&var_name, Value::Vec(vec))?;
                            Ok(new_val)
                        } else {
                            Err(PulseError::InvalidAssignment(
                                "Unable to determine variable for assignment".into(),
                                access.base.span(),
                            )
                                .into())
                        }
                    } else {
                        Err(PulseError::TypeMismatch(
                            "Left side of assignment must be a vector with integer index"
                                .into(),
                            access.base.span(),
                        )
                            .into())
                    }
                }
            },
            _ => todo!("missing left: {:?}", left),
        }
    }

    /// Access a field of a value.
    ///
    /// # Arguments
    /// * `value` - The [Value] to access the field of.
    /// * `expr` - The [Expr] representing the field to access.
    /// * `ctx` - The context in which to access the field.
    ///
    /// # Returns
    /// The value of the field.
    pub fn access_field(&mut self, value: Value, expr: &Expr, ctx: &Context) -> Result<Value> {
        match expr {
            Expr::Call(call) => {
                let methods = value.builtin_methods();
                if let Some(method) = methods.get(&call.callee) {
                    let mut args = vec![value.clone()];
                    for arg in call.args.iter() {
                        self.interpret_expr(arg, ctx)?;
                        args.push(self.vm.pop().expect("Expected value on stack"));
                    }

                    method.clone().call(args)
                } else {
                    Err(PropertyNotFoundError(call.callee.clone(), expr.span()).into())
                }
            }
            Expr::Literal(lit) => {
                if let LiteralType::String(s) = &lit.value {
                    unimplemented!("There is not future that requires this code to be implemented now. This will be implemented with objects/structs.");
                    // self.access_field(&Expr::Literal(lit.clone()))
                } else {
                    Err(PropertyNotFoundError("".to_string(), expr.span()).into())
                }
            }
            _ => {
                self.interpret_expr(expr, ctx)?;

                let field = self.vm.pop().expect("Expected value on stack");

                Ok(field)
            }
        }
    }
}