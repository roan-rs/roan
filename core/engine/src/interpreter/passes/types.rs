use crate::{context::Context, interpreter::passes::Pass, module::Module, vm::VM};
use anyhow::Result;
use roan_ast::{Expr, GetSpan, LiteralType, Stmt, TypeAnnotation, UnOpKind};
use roan_error::error::RoanError::TypeMismatch;

#[derive(Clone)]
pub struct TypePass;

impl Pass for TypePass {
    fn pass_stmt(
        &mut self,
        stmt: Stmt,
        module: &mut Module,
        ctx: &mut Context,
        vm: &mut VM,
    ) -> Result<()> {
        match stmt {
            Stmt::Fn(mut func) => {
                for param in func.params.iter_mut() {
                    self.check_type_annotation(&mut param.type_annotation, module, ctx)?;
                }
                self.validate_function(&mut func, module, ctx)?;
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedType {
    Int,
    Float,
    Bool,
    String,
    Char,
    // Name of a struct - defining module
    Struct(String, String),
    Null,
    // Object value type can be any type
    Object(Box<ResolvedType>),
    Vector(Box<ResolvedType>),
    Any,
}

impl ResolvedType {
    pub fn matches(type1: ResolvedType, type2: ResolvedType) -> bool {
        match (type1, type2) {
            (ResolvedType::Int, ResolvedType::Float)
            | (ResolvedType::Float, ResolvedType::Int)
            | (ResolvedType::Int, ResolvedType::Int)
            | (ResolvedType::Float, ResolvedType::Float) => true,
            (ResolvedType::String, ResolvedType::Char)
            | (ResolvedType::Char, ResolvedType::String)
            | (ResolvedType::String, ResolvedType::String) => true,
            (
                ResolvedType::Struct(name1, def_module1),
                ResolvedType::Struct(name2, def_module2),
            ) => name1 == name2 && def_module1 == def_module2,
            (ResolvedType::Vector(type1), ResolvedType::Vector(type2))
            | (ResolvedType::Object(type1), ResolvedType::Object(type2)) => {
                ResolvedType::matches(*type1, *type2)
            }
            (ResolvedType::Any, _) | (_, ResolvedType::Any) => true,
            _ => false,
        }
    }

    pub fn to_type_annotation(&self) -> TypeAnnotation {
        TypeAnnotation {
            separator: None,
            token_name: None,

            type_name: "".to_string(),
            is_array: matches!(self, ResolvedType::Vector(_)),
            is_nullable: false,
            module_id: None,
            is_generic: false,
            generics: vec![],
        }
    }

    pub fn from_type_annotation(typ: &TypeAnnotation) -> ResolvedType {
        match typ.type_name.as_str() {
            "int" => ResolvedType::Int,
            "float" => ResolvedType::Float,
            "bool" => ResolvedType::Bool,
            "string" => ResolvedType::String,
            "char" => ResolvedType::Char,
            "null" => ResolvedType::Null,
            "object" => ResolvedType::Object(Box::new(ResolvedType::from_type_annotation(
                &typ.generics[0],
            ))),
            "vec" => ResolvedType::Vector(Box::new(ResolvedType::from_type_annotation(
                &typ.generics[0],
            ))),
            "anytype" => ResolvedType::Any,
            _ => ResolvedType::Any,
        }
    }

    pub fn matches_to(type1: ResolvedType, type2: ResolvedType, to: ResolvedType) -> bool {
        ResolvedType::matches(type1, to.clone()) && ResolvedType::matches(type2, to)
    }
}

impl TypePass {
    pub fn check_type_annotation(
        &self,
        typ: &mut TypeAnnotation,
        module: &mut Module,
        ctx: &mut Context,
    ) -> Result<()> {
        typ.module_id = Some(module.id().clone());

        if typ.is_generic && !typ.generics.is_empty() {
            for generic in typ.generics.iter_mut() {
                self.check_type_annotation(generic, module, ctx)?;
            }
        }

        Ok(())
    }

    /// Checks if an annotation is a valid type and if the return statement of a function is valid.
    pub fn validate_function(
        &self,
        func: &mut roan_ast::Fn,
        module: &mut Module,
        ctx: &mut Context,
    ) -> Result<()> {
        if let Some(typ) = &mut func.return_type {
            self.check_type_annotation(typ, module, ctx)?;
        }

        self.validate_block(&func.body.stmts, module, ctx)?;

        Ok(())
    }

    pub fn validate_and_get_type_expr(
        &self,
        expr: &Expr,
        module: &mut Module,
        ctx: &mut Context,
        global_type: Option<TypeAnnotation>,
    ) -> Result<ResolvedType> {
        match expr {
            Expr::Literal(lit) => match lit.value {
                LiteralType::String(_) => Ok(ResolvedType::String),
                LiteralType::Int(_) => Ok(ResolvedType::Int),
                LiteralType::Float(_) => Ok(ResolvedType::Float),
                LiteralType::Bool(_) => Ok(ResolvedType::Bool),
                LiteralType::Null => Ok(ResolvedType::Null),
                LiteralType::Char(_) => Ok(ResolvedType::Char),
            },
            Expr::Null(_) => Ok(ResolvedType::Null),
            Expr::Object(obj) => {
                let accepts_any = global_type
                    .as_ref()
                    .map(|gt| gt.is_generic("object", vec!["anytype"]))
                    .unwrap_or(false);

                let mut obj_type = ResolvedType::Null;
                for (key, value) in &obj.fields {
                    let value_type =
                        self.validate_and_get_type_expr(value, module, ctx, global_type.clone())?;
                    if obj_type == ResolvedType::Null {
                        obj_type = value_type;
                    } else if !ResolvedType::matches(obj_type.clone(), value_type) && !accepts_any {
                        return Err(TypeMismatch(
                            "All fields of an object must have the same type".to_string(),
                            value.span().clone(),
                        )
                        .into());
                    }
                    if accepts_any {
                        obj_type = ResolvedType::Any;
                    }
                }
                Ok(ResolvedType::Object(Box::new(obj_type)))
            }
            Expr::ThenElse(then_else) => {
                let then_type = self.validate_and_get_type_expr(
                    &then_else.then_expr,
                    module,
                    ctx,
                    global_type.clone(),
                )?;
                let else_type = self.validate_and_get_type_expr(
                    &then_else.else_expr,
                    module,
                    ctx,
                    global_type.clone(),
                )?;

                if let Some(typ) = &global_type {
                    if typ.is_any()
                        || ResolvedType::matches_to(
                            then_type.clone(),
                            else_type.clone(),
                            ResolvedType::from_type_annotation(typ),
                        )
                    {
                        Ok(ResolvedType::from_type_annotation(typ))
                    } else {
                        Err(TypeMismatch(
                            format!("Both branches of a then-else expression must match type annotation: {}", typ.type_name),
                            then_else.span().clone(),
                        ).into())
                    }
                } else if ResolvedType::matches(then_type.clone(), else_type.clone()) {
                    Ok(then_type)
                } else {
                    Err(TypeMismatch(
                        "Two branches of a then-else expression must have the same type"
                            .to_string(),
                        then_else.span().clone(),
                    )
                    .into())
                }
            }
            Expr::Vec(vec) => {
                let accepts_any = global_type
                    .as_ref()
                    .map(|gt| gt.is_generic("vec", vec!["anytype"]))
                    .unwrap_or(false);

                let mut vec_type = ResolvedType::Null;
                for expr in &vec.exprs {
                    let expr_type =
                        self.validate_and_get_type_expr(expr, module, ctx, global_type.clone())?;
                    if vec_type == ResolvedType::Null {
                        vec_type = expr_type;
                    } else if let Some(ref typ) = global_type {
                        if !ResolvedType::matches_to(
                            vec_type.clone(),
                            expr_type.clone(),
                            ResolvedType::from_type_annotation(&typ.generics[0]),
                        ) {
                            return Err(TypeMismatch(
                                format!(
                                    "All elements of a vector must match type annotation: {}",
                                    typ.type_name
                                ),
                                expr.span().clone(),
                            )
                            .into());
                        }
                    } else if !ResolvedType::matches(vec_type.clone(), expr_type.clone())
                        && !accepts_any
                    {
                        return Err(TypeMismatch(
                            "All elements of a vector must have the same type".to_string(),
                            expr.span().clone(),
                        )
                        .into());
                    }
                    if accepts_any {
                        vec_type = ResolvedType::Any;
                    }
                }
                Ok(ResolvedType::Vector(Box::new(vec_type)))
            }
            Expr::Unary(unary) => match unary.operator.kind {
                UnOpKind::Minus | UnOpKind::BitwiseNot => {
                    let expr_type =
                        self.validate_and_get_type_expr(&unary.expr, module, ctx, global_type)?;
                    if expr_type == ResolvedType::Int || expr_type == ResolvedType::Float {
                        Ok(expr_type)
                    } else {
                        Err(TypeMismatch(
                            format!(
                                "Unary operator {} can only be applied to int or float",
                                unary.operator.kind
                            ),
                            unary.span().clone(),
                        )
                        .into())
                    }
                }
                UnOpKind::LogicalNot => {
                    // Validate the expression but do not enforce type checking, allowing null value checks
                    self.validate_and_get_type_expr(&unary.expr, module, ctx, global_type)?;
                    Ok(ResolvedType::Bool)
                }
            },
            _ => Ok(ResolvedType::Null),
        }
    }

    pub fn validate_block(
        &self,
        block: &Vec<Stmt>,
        module: &mut Module,
        ctx: &mut Context,
    ) -> Result<()> {
        for stmt in block.iter() {
            match stmt {
                Stmt::Return(ret) if ret.expr.is_some() => {}
                Stmt::Block(block) => {
                    self.validate_block(&block.stmts, module, ctx)?;
                }
                Stmt::If(if_stmt) => {
                    self.validate_block(&if_stmt.then_block.stmts, module, ctx)?;

                    if if_stmt.else_ifs.len() > 0 {
                        for else_if in if_stmt.else_ifs.iter() {
                            self.validate_block(&else_if.block.stmts, module, ctx)?;
                        }
                    }

                    if let Some(else_branch) = &if_stmt.else_block {
                        self.validate_block(&else_branch.block.stmts, module, ctx)?;
                    }
                }
                Stmt::While(while_stmt) => {
                    self.validate_block(&while_stmt.block.stmts, module, ctx)?;
                }
                Stmt::Loop(loop_stmt) => {
                    self.validate_block(&loop_stmt.block.stmts, module, ctx)?;
                }
                // We just validate all types of expressions
                Stmt::Expr(expr) => {
                    self.validate_and_get_type_expr(expr.as_ref(), module, ctx, None)?;
                }
                Stmt::Let(let_stmt) => {
                    let mut let_stmt = let_stmt.clone();

                    if let Some(mut typ) = let_stmt.type_annotation.clone() {
                        self.check_type_annotation(&mut typ, module, ctx)?;

                        let mut expr_type = self
                            .validate_and_get_type_expr(
                                let_stmt.initializer.as_ref(),
                                module,
                                ctx,
                                Some(typ),
                            )?
                            .to_type_annotation();

                        self.check_type_annotation(&mut expr_type, module, ctx)?;
                    } else {
                        let typ = self
                            .validate_and_get_type_expr(
                                let_stmt.initializer.as_ref(),
                                module,
                                ctx,
                                let_stmt.type_annotation,
                            )?
                            .to_type_annotation();

                        let mut typ_clone = typ.clone();
                        self.check_type_annotation(&mut typ_clone, module, ctx)?;

                        let_stmt.type_annotation = Some(typ);
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
