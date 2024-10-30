use crate::{context::Context, module::Module, value::Value, vm::VM};
use anyhow::Result;
use roan_ast::{AccessExpr, AccessKind, Expr, GetSpan};
use roan_error::error::PulseError::{
    PropertyNotFoundError, StaticContext, StaticMemberAccess, UndefinedFunctionError,
};

impl Module {
    /// Interpret an access expression.
    ///
    /// # Arguments
    /// * `access` - [AccessExpr] expression to interpret.
    /// * `ctx` - The context in which to interpret the access expression.
    /// * `vm` - The virtual machine to use.
    ///
    /// # Returns
    /// The result of the access expression.
    pub fn interpret_access(
        &mut self,
        access: AccessExpr,
        ctx: &mut Context,
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
                                self.interpret_expr(arg, ctx, vm)?;
                                Ok(vm.pop().unwrap())
                            })
                            .collect::<Result<Vec<_>>>()?;

                        let mut def_module = ctx.query_module(&struct_def.defining_module).unwrap();

                        self.execute_user_defined_function(
                            method.clone(),
                            &mut def_module,
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
        ctx: &mut Context,
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

                    let mut def_module = ctx.query_module(&struct_def.defining_module).unwrap();

                    self.execute_user_defined_function(
                        field.clone(),
                        &mut def_module,
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
                    Value::Object(fields) => {
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
