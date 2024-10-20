use crate::{
    context::Context,
    module::{Module, StoredFunction},
    value::Value,
    vm::VM,
};
use anyhow::Result;
use log::debug;
use roan_ast::{
    AccessExpr, AccessKind, Assign, AssignOperator, BinOpKind, Binary, CallExpr, Expr, GetSpan,
    UnOpKind, Unary, VecExpr,
};
use roan_error::{
    error::{
        PulseError,
        PulseError::{
            InvalidSpread, PropertyNotFoundError, StaticContext, StaticMemberAccess,
            StaticMemberAssignment, UndefinedFunctionError, VariableNotFoundError,
        },
    },
    print_diagnostic,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
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
    pub fn interpret_expr(&mut self, expr: &Expr, ctx: &Context, vm: &mut VM) -> Result<()> {
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
            _ => todo!("missing expr: {:?}", expr),
        };

        Ok(vm.push(val?))
    }

    /// Interpret a struct constructor expression.
    ///
    /// # Arguments
    /// * `constructor` - [StructConstructor] expression to interpret.
    /// * `ctx` - The context in which to interpret the struct constructor expression.
    /// * `vm` - The virtual machine to use.
    ///
    /// # Returns
    /// The result of the struct constructor expression.
    pub fn interpret_struct_constructor(
        &mut self,
        constructor: roan_ast::StructConstructor,
        ctx: &Context,
        vm: &mut VM,
    ) -> Result<Value> {
        debug!("Interpreting struct constructor");
        let struct_def = self.get_struct(&constructor.name, constructor.token.span.clone())?;

        let mut fields = HashMap::new();

        for (field_name, expr) in constructor.fields.iter() {
            self.interpret_expr(expr, ctx, vm)?;
            fields.insert(field_name.clone(), vm.pop().unwrap());
        }

        Ok(Value::Struct(struct_def, fields))
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
    pub fn interpret_unary(&mut self, u: Unary, ctx: &Context, vm: &mut VM) -> Result<Value> {
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
                return Err(PulseError::InvalidUnaryOperation(
                    u.operator.kind.to_string(),
                    u.span(),
                )
                .into())
            }
        };

        Ok(val)
    }

    /// Interpret an access expression.
    ///
    /// # Arguments
    /// * `access` - [Access] expression to interpret.
    /// * `ctx` - The context in which to interpret the access expression.
    /// * `vm` - The virtual machine to use.
    ///
    /// # Returns
    /// The result of the access expression.
    pub fn interpret_access(
        &mut self,
        access: AccessExpr,
        ctx: &Context,
        vm: &mut VM,
    ) -> Result<Value> {
        match access.access.clone() {
            AccessKind::Field(field_expr) => {
                let base = access.base.clone();

                self.interpret_expr(&base, ctx, vm)?;
                let base = vm.pop().unwrap();

                Ok(self.access_field(base, &field_expr, ctx, vm)?)
            }
            AccessKind::Index(index_expr) => {
                self.interpret_expr(&index_expr, ctx, vm)?;
                let index = vm.pop().unwrap();

                self.interpret_expr(&access.base, ctx, vm)?;
                let base = vm.pop().unwrap();

                Ok(base.access_index(index))
            }
            AccessKind::StaticMethod(expr) => {
                let base = access.base.as_ref().clone();

                let (struct_name, span) = match base {
                    Expr::Variable(v) => (v.ident.clone(), v.token.span.clone()),
                    _ => return Err(StaticMemberAccess(access.span()).into()),
                };

                let struct_def = self.get_struct(&struct_name, span)?;

                let expr = expr.as_ref().clone();
                match expr {
                    Expr::Call(call) => {
                        let method_name = call.callee.clone();
                        let method = struct_def.find_static_method(&method_name);

                        if method.is_none() {
                            return Err(UndefinedFunctionError(
                                method_name,
                                call.token.span.clone(),
                            )
                            .into());
                        }

                        let method = method.unwrap();

                        let args = call
                            .args
                            .iter()
                            .map(|arg| {
                                self.interpret_expr(arg, ctx, vm).unwrap();
                                vm.pop().unwrap()
                            })
                            .collect();

                        self.execute_user_defined_function(
                            method.clone(),
                            Arc::new(Mutex::new(self.clone())),
                            args,
                            ctx,
                            vm,
                            &call,
                        )?;

                        Ok(vm.pop().unwrap())
                    }
                    _ => return Err(StaticContext(expr.span()).into()),
                }
            }
        }
    }

    /// Interpret a then-else expression.
    ///
    /// # Arguments
    /// * `then_else` - [ThenElse] expression to interpret.
    /// * `ctx` - The context in which to interpret the then-else expression.
    /// * `vm` - The virtual machine to use.
    ///
    /// # Returns
    /// The result of the then-else expression.
    pub fn interpret_then_else(
        &mut self,
        then_else: roan_ast::ThenElse,
        ctx: &Context,
        vm: &mut VM,
    ) -> Result<Value> {
        debug!("Interpreting then-else");

        self.interpret_expr(&then_else.condition, ctx, vm)?;
        let condition = vm.pop().unwrap();

        let b = match condition {
            Value::Bool(b) => b,
            _ => condition.is_truthy(),
        };

        if b {
            self.interpret_expr(&then_else.then_expr, ctx, vm)?;
        } else {
            self.interpret_expr(&then_else.else_expr, ctx, vm)?;
        }

        Ok(vm.pop().expect("Expected value on stack"))
    }

    /// Interpret a vector expression.
    ///
    /// # Arguments
    /// * `vec` - [VecExpr] to interpret.
    /// * `ctx` - The context in which to interpret the vector expression.
    ///
    /// # Returns
    /// The result of the vector expression.
    pub fn interpret_vec(&mut self, vec: VecExpr, ctx: &Context, vm: &mut VM) -> Result<Value> {
        debug!("Interpreting vec: {:?}", vec);

        let mut values = vec![];

        for expr in vec.exprs.iter() {
            match expr {
                Expr::Spread(s) => {
                    self.interpret_expr(&s.expr, ctx, vm)?;
                    let spread_val = vm.pop().unwrap();

                    if let Value::Vec(vec) = spread_val {
                        values.extend(vec);
                    } else {
                        return Err(InvalidSpread(s.expr.span()).into());
                    }
                }
                _ => {
                    self.interpret_expr(expr, ctx, vm)?;
                    values.push(vm.pop().unwrap());
                }
            }
        }

        Ok(Value::Vec(values))
    }

    /// Interpret a call expression.
    ///
    /// # Arguments
    /// * `call` - [CallExpr] to interpret.
    /// * `ctx` - The context in which to interpret the call.
    ///
    /// # Returns
    /// The result of the call.
    pub fn interpret_call(&mut self, call: &CallExpr, ctx: &Context, vm: &mut VM) -> Result<Value> {
        debug!("Interpreting call: {:?}", call);

        let mut args = vec![];
        for arg in call.args.iter() {
            match arg {
                Expr::Spread(s) => {
                    self.interpret_expr(&s.expr, ctx, vm)?;
                    let spread_val = vm.pop().unwrap();

                    if let Value::Vec(vec) = spread_val {
                        args.extend(vec);
                    } else {
                        return Err(InvalidSpread(s.expr.span()).into());
                    }
                }
                _ => {
                    self.interpret_expr(arg, ctx, vm)?;
                    args.push(vm.pop().unwrap());
                }
            }
        }

        let stored_function = self
            .find_function(&call.callee)
            .ok_or_else(|| UndefinedFunctionError(call.callee.clone(), call.token.span.clone()))?
            .clone();

        match stored_function {
            StoredFunction::Native(n) => {
                self.execute_native_function(n, args, vm)?;

                Ok(vm.pop().unwrap())
            }
            StoredFunction::Function {
                function,
                defining_module,
            } => {
                match self.execute_user_defined_function(
                    function,
                    defining_module.clone(),
                    args,
                    ctx,
                    vm,
                    call,
                ) {
                    Ok(_) => Ok(vm.pop().unwrap_or(Value::Void)),
                    Err(e) => {
                        print_diagnostic(e, Some(defining_module.lock().unwrap().source.content()));
                        std::process::exit(1);
                    }
                }
            }
        }
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
        ctx: &Context,
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

            // TODO: add more bitwise operators
            (Value::Int(a), BinOpKind::BitwiseAnd, Value::Int(b)) => Value::Int(a & b),
            (Value::Int(a), BinOpKind::BitwiseOr, Value::Int(b)) => Value::Int(a | b),
            (Value::Int(a), BinOpKind::BitwiseXor, Value::Int(b)) => Value::Int(a ^ b),
            (Value::Int(a), BinOpKind::ShiftLeft, Value::Int(b)) => Value::Int(a << b),
            (Value::Int(a), BinOpKind::ShiftRight, Value::Int(b)) => Value::Int(a >> b),

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
                    let base = access.base.clone();

                    self.interpret_expr(right, ctx, vm)?;
                    let new_val = vm.pop().unwrap();
                    unimplemented!("field access")
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

    /// Access a field of a value.
    ///
    /// # Arguments
    /// * `value` - The [Value] to access the field of.
    /// * `expr` - The [Expr] representing the field to access.
    /// * `ctx` - The context in which to access the field.
    ///
    /// # Returns
    /// The value of the field.
    pub fn access_field(
        &mut self,
        value: Value,
        expr: &Expr,
        ctx: &Context,
        vm: &mut VM,
    ) -> Result<Value> {
        match expr {
            Expr::Call(call) => {
                let value_clone = value.clone();
                if let Value::Struct(struct_def, _) = value_clone {
                    let field = struct_def.find_method(&call.callee);

                    if field.is_none() {
                        return Err(PropertyNotFoundError(call.callee.clone(), expr.span()).into());
                    }

                    let field = field.unwrap();
                    let mut args = vec![value.clone()];
                    for arg in call.args.iter() {
                        self.interpret_expr(arg, ctx, vm)?;
                        args.push(vm.pop().expect("Expected value on stack"));
                    }

                    self.execute_user_defined_function(
                        field.clone(),
                        Arc::new(Mutex::new(self.clone())),
                        args,
                        ctx,
                        vm,
                        call,
                    )?;

                    return Ok(vm.pop().expect("Expected value on stack"));
                }

                let methods = value.builtin_methods();
                if let Some(method) = methods.get(&call.callee) {
                    let mut args = vec![value.clone()];
                    for arg in call.args.iter() {
                        self.interpret_expr(arg, ctx, vm)?;
                        args.push(vm.pop().expect("Expected value on stack"));
                    }

                    self.execute_native_function(method.clone(), args, vm)?;

                    Ok(vm.pop().expect("Expected value on stack"))
                } else {
                    Err(PropertyNotFoundError(call.callee.clone(), expr.span()).into())
                }
            }
            Expr::Variable(lit) => {
                let name = lit.ident.clone();
                match value {
                    Value::Struct(_, fields) => {
                        let field = fields.get(&name).ok_or_else(|| {
                            PropertyNotFoundError(name.clone(), lit.token.span.clone())
                        })?;

                        Ok(field.clone())
                    }
                    _ => Err(PropertyNotFoundError(name.clone(), lit.token.span.clone()).into()),
                }
            }
            _ => {
                self.interpret_expr(expr, ctx, vm)?;

                let field = vm.pop().expect("Expected value on stack");

                Ok(field)
            }
        }
    }
}
