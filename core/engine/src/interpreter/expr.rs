use crate::{context::Context, module::Module, value::Value, vm::VM};
use anyhow::Result;
use indexmap::IndexMap;
use log::debug;
use roan_ast::{
    AccessKind, Assign, AssignOperator, BinOpKind, Binary, Expr, GetSpan, LiteralType, Spread,
    UnOpKind, Unary, VecExpr,
};
use roan_error::error::{
    RoanError,
    RoanError::{InvalidSpread, StaticMemberAssignment, VariableNotFoundError},
};

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
    pub fn interpret_expr(&mut self, expr: &Expr, ctx: &mut Context, vm: &mut VM) -> Result<()> {
        let val: Result<Value> = match expr {
            Expr::Variable(v) => {
                debug!("Interpreting variable: {}", v.ident);

                let variable: &Value = self
                    .find_variable(&v.ident)
                    .or_else(|| {
                        let constant = self.find_const(&v.ident);

                        if let Some(constant) = constant {
                            Some(&constant.value)
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| VariableNotFoundError(v.ident.clone(), v.token.span.clone()))?;

                Ok(variable.clone())
            }
            Expr::Literal(l) => {
                debug!("Interpreting literal: {:?}", l);

                Ok(Value::from_literal(l.clone()))
            }
            Expr::Call(call) => self.interpret_call(call, ctx, vm),
            Expr::Parenthesized(p) => {
                debug!("Interpreting parenthesized: {:?}", p);

                self.interpret_expr(&p.expr, ctx, vm)?;

                Ok(vm.pop().unwrap())
            }
            Expr::Access(access) => self.interpret_access(access.clone(), ctx, vm),
            Expr::StructConstructor(constructor) => {
                self.interpret_struct_constructor(constructor.clone(), ctx, vm)
            }
            Expr::Assign(assign) => self.interpret_assignment(assign.clone(), ctx, vm),
            Expr::Vec(vec) => self.interpret_vec(vec.clone(), ctx, vm),
            Expr::Binary(b) => self.interpret_binary(b.clone(), ctx, vm),
            // Spread operator are only supposed to be used in vectors and function calls
            Expr::Spread(s) => Err(InvalidSpread(s.expr.span()).into()),
            Expr::Null(_) => Ok(Value::Null),
            Expr::Unary(u) => self.interpret_unary(u.clone(), ctx, vm),
            Expr::ThenElse(then_else) => self.interpret_then_else(then_else.clone(), ctx, vm),
            Expr::Object(obj) => {
                let mut fields = IndexMap::new();

                for (field_name, expr) in obj.fields.iter() {
                    self.interpret_expr(expr, ctx, vm)?;
                    fields.insert(field_name.clone(), vm.pop().unwrap());
                }

                Ok(Value::Object(fields))
            }
        };

        Ok(vm.push(val?))
    }

    /// Interpret a unary expression.
    ///
    /// # Arguments
    /// * `unary` - [Unary] expression to interpret.
    /// * `ctx` - The context in which to interpret the unary expression.
    /// * `vm` - The virtual machine to use.
    ///
    /// # Returns
    /// The result of the unary expression.
    pub fn interpret_unary(&mut self, u: Unary, ctx: &mut Context, vm: &mut VM) -> Result<Value> {
        self.interpret_expr(&u.expr, ctx, vm)?;
        let val = vm.pop().unwrap();

        let val = match (u.operator.clone().kind, val.clone()) {
            (UnOpKind::Minus, Value::Int(i)) => Value::Int(-i),
            (UnOpKind::Minus, Value::Float(f)) => Value::Float(-f),
            (UnOpKind::LogicalNot, Value::Bool(b)) => Value::Bool(!b),
            (UnOpKind::BitwiseNot, Value::Int(i)) => Value::Int(!i),
            (UnOpKind::LogicalNot, Value::Null) => Value::Bool(true),
            (UnOpKind::LogicalNot, _) => {
                let b = val.is_truthy();

                Value::Bool(!b)
            }
            _ => {
                return Err(
                    RoanError::InvalidUnaryOperation(u.operator.kind.to_string(), u.span()).into(),
                )
            }
        };

        Ok(val)
    }

    /// Interpret a vector expression.
    ///
    /// # Arguments
    /// * `vec` - [VecExpr] to interpret.
    /// * `ctx` - The context in which to interpret the vector expression.
    ///
    /// # Returns
    /// The result of the vector expression.
    pub fn interpret_vec(&mut self, vec: VecExpr, ctx: &mut Context, vm: &mut VM) -> Result<Value> {
        debug!("Interpreting vec: {:?}", vec);

        Ok(Value::Vec(
            self.interpret_possible_spread(vec.exprs, ctx, vm)?,
        ))
    }

    /// Interpret a binary expression.
    ///
    /// # Arguments
    /// * `binary_expr` - [Binary] expression to interpret.
    /// * `ctx` - The context in which to interpret the binary expression.
    ///
    /// # Returns
    /// The result of the binary expression.
    pub fn interpret_binary(
        &mut self,
        binary_expr: Binary,
        ctx: &mut Context,
        vm: &mut VM,
    ) -> Result<Value> {
        debug!("Interpreting binary: {:?}", binary_expr);

        self.interpret_expr(&binary_expr.left, ctx, vm)?;
        let left = vm.pop().unwrap();
        self.interpret_expr(&binary_expr.right, ctx, vm)?;
        let right = vm.pop().unwrap();

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

            (Value::Int(a), BinOpKind::BitwiseAnd, Value::Int(b)) => Value::Int(a & b),
            (Value::Int(a), BinOpKind::BitwiseOr, Value::Int(b)) => Value::Int(a | b),
            (Value::Int(a), BinOpKind::BitwiseXor, Value::Int(b)) => Value::Int(a ^ b),
            (Value::Int(a), BinOpKind::ShiftLeft, Value::Int(b)) => Value::Int(a << b),
            (Value::Int(a), BinOpKind::ShiftRight, Value::Int(b)) => Value::Int(a >> b),

            _ => todo!("missing binary operator: {:?}", binary_expr.operator),
        };

        Ok(val)
    }

    /// Interpret a spread expression.
    ///
    /// This function requires vec of values to push to.
    ///
    /// # Arguments
    /// * `expr` - [Expr] to interpret.
    /// * `ctx` - The context in which to interpret the expression.
    /// * `values` - The vec of values to push to.
    ///
    /// # Returns
    /// The result of the expression.
    pub fn interpret_spread(
        &mut self,
        s: Spread,
        ctx: &mut Context,
        vm: &mut VM,
        values: &mut Vec<Value>,
    ) -> Result<()> {
        self.interpret_expr(&s.expr, ctx, vm)?;
        let spread_val = vm.pop().unwrap();

        if let Value::Vec(vec) = spread_val {
            values.extend(vec);
        } else {
            return Err(InvalidSpread(s.expr.span()).into());
        }

        Ok(())
    }

    /// Helper function to interpret possible spread expressions.
    ///
    /// # Arguments
    /// * `exprs` - The expressions to interpret.
    /// * `ctx` - The context in which to interpret the expressions.
    /// * `vm` - The virtual machine to use.
    pub fn interpret_possible_spread(
        &mut self,
        exprs: Vec<Expr>,
        ctx: &mut Context,
        vm: &mut VM,
    ) -> Result<Vec<Value>> {
        let mut values = vec![];

        for expr in exprs.iter() {
            match expr {
                Expr::Spread(s) => self.interpret_spread(s.clone(), ctx, vm, &mut values)?,
                _ => {
                    self.interpret_expr(expr, ctx, vm)?;
                    values.push(vm.pop().unwrap());
                }
            }
        }

        Ok(values)
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
        ctx: &mut Context,
        vm: &mut VM,
    ) -> Result<Value> {
        debug!("Interpreting assign: {:?}", assign);
        let left = assign.left.as_ref();
        let right = assign.right.as_ref();
        let operator = assign.op.clone();

        match left {
            Expr::Variable(v) => {
                self.interpret_expr(right, ctx, vm)?;
                let val = vm.pop().unwrap();
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
                    let base = access.base.as_ref().clone();

                    self.interpret_expr(right, ctx, vm)?;
                    let new_val = vm.pop().unwrap();

                    self.interpret_expr(&base, ctx, vm)?;
                    let base_val = vm.pop().unwrap();

                    let field_name = {
                        let field = field.as_ref().clone();
                        match field {
                            Expr::Variable(v) => v.ident.clone(),
                            Expr::Literal(l) => match l.value.clone() {
                                LiteralType::String(s) => s,
                                _ => panic!("Invalid field name"),
                            },
                            // TODO: error
                            _ => panic!("Invalid field name"),
                        }
                    };

                    let var_name = Self::extract_variable_name(&access.base);
                    
                    if var_name.is_none() {
                        return Err(RoanError::InvalidAssignment(
                            "Unable to determine variable for assignment".into(),
                            access.base.span(),
                        ).into())
                    }
                    
                    let var_name = var_name.unwrap();

                    match base_val {
                        Value::Object(mut fields) => {
                            fields.insert(field_name, new_val.clone());
                            self.set_variable(&var_name, Value::Object(fields))?;

                            Ok(new_val)
                        }
                        Value::Struct(def, mut fields) => {
                            if def.fields.get(&field_name).is_none() {
                                return Err(RoanError::PropertyAssignmentError(
                                    field_name,
                                    access.span(),
                                )
                                .into());
                            }

                            fields.insert(field_name, new_val.clone());
                            self.set_variable(&var_name, Value::Struct(def, fields))?;

                            Ok(new_val)
                        }
                        _ => Err(RoanError::TypeMismatch(
                            "Left side of assignment must be a struct or object".into(),
                            access.base.span(),
                        )
                        .into()),
                    }
                }
                AccessKind::Index(index_expr) => {
                    self.interpret_expr(&access.base, ctx, vm)?;
                    let base_val = vm.pop().unwrap();

                    self.interpret_expr(index_expr, ctx, vm)?;
                    let index_val = vm.pop().unwrap();

                    self.interpret_expr(right, ctx, vm)?;
                    let new_val = vm.pop().unwrap();

                    if let (Value::Vec(mut vec), Value::Int(index)) = (base_val.clone(), index_val)
                    {
                        let idx = index as usize;
                        if idx >= vec.len() {
                            return Err(RoanError::IndexOutOfBounds(
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
                            Err(RoanError::InvalidAssignment(
                                "Unable to determine variable for assignment".into(),
                                access.base.span(),
                            )
                            .into())
                        }
                    } else {
                        Err(RoanError::TypeMismatch(
                            "Left side of assignment must be a vector with integer index".into(),
                            access.base.span(),
                        )
                        .into())
                    }
                }
                AccessKind::StaticMethod(_) => Err(StaticMemberAssignment(access.span()).into()),
            },
            _ => todo!("missing left: {:?}", left),
        }
    }
}
