use crate::{context::Context, interpreter::passes::Pass, module::Module, value::Value, vm::VM};
use anyhow::Result;
use colored::Colorize;
use indexmap::IndexMap;
use roan_ast::{
    AssignOperator, BinOpKind, Expr, GetSpan, LiteralType, Stmt, TypeAnnotation, UnOpKind,
};
use roan_error::{
    error::RoanError::{MissingField, TypeMismatch, VariableNotFoundError},
    TextSpan,
};
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};
use tracing::debug;

#[derive(Clone)]
pub struct TypePass {
    pub scopes: Vec<HashMap<String, ResolvedType>>,
}

impl TypePass {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn declare_variable(&mut self, name: String, typ: ResolvedType) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, typ);
        }
    }

    pub fn set_variable(&mut self, name: &str, val: ResolvedType) -> Result<()> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), val);
                return Ok(());
            }
        }
        Err(VariableNotFoundError(name.to_string(), TextSpan::default()).into())
    }

    pub fn find_variable(&self, name: &str) -> Option<&ResolvedType> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val);
            }
        }
        None
    }
}

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
            _ => self.validate_stmt(&stmt, module, ctx)?,
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

impl Display for ResolvedType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedType::Int => write!(f, "int"),
            ResolvedType::Float => write!(f, "float"),
            ResolvedType::Bool => write!(f, "bool"),
            ResolvedType::String => write!(f, "string"),
            ResolvedType::Char => write!(f, "char"),
            ResolvedType::Struct(name, _) => write!(f, "{}", name),
            ResolvedType::Null => write!(f, "null"),
            ResolvedType::Object(t) => write!(f, "object<{}>", t),
            ResolvedType::Vector(t) => write!(f, "vec<{}>", t),
            ResolvedType::Any => write!(f, "any"),
        }
    }
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
        let generics = match self {
            ResolvedType::Object(t) | ResolvedType::Vector(t) => vec![t.to_type_annotation()],
            _ => vec![],
        };

        TypeAnnotation {
            separator: None,
            token_name: None,
            type_name: match self {
                ResolvedType::Int => "int".to_string(),
                ResolvedType::Float => "float".to_string(),
                ResolvedType::Bool => "bool".to_string(),
                ResolvedType::String => "string".to_string(),
                ResolvedType::Char => "char".to_string(),
                ResolvedType::Struct(name, _) => name.clone(),
                ResolvedType::Null => "null".to_string(),
                ResolvedType::Object(t) => "object".to_string(),
                ResolvedType::Vector(t) => "vec".to_string(),
                ResolvedType::Any => "anytype".to_string(),
            },
            is_array: matches!(self, ResolvedType::Vector(_)),
            is_nullable: false,
            module_id: None,
            is_generic: generics.len() > 0,
            generics,
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
        &mut self,
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
            Expr::Null(_) => Ok(ResolvedType::Null),
            Expr::Parenthesized(expr) => {
                self.validate_and_get_type_expr(&expr.expr, module, ctx, global_type)
            }
            Expr::StructConstructor(constructor) => {
                let struct_type =
                    module.get_struct(&constructor.name, constructor.token.span.clone())?;

                for (name, field) in &struct_type.fields {
                    let constructor_field =
                        constructor.fields.iter().find(|(n, _)| n.clone() == name);

                    if let Some((name, expr)) = constructor_field {
                        let expr_type = self.validate_and_get_type_expr(
                            expr,
                            module,
                            ctx,
                            Some(field.type_annotation.clone()),
                        )?;

                        if !ResolvedType::matches(
                            expr_type,
                            ResolvedType::from_type_annotation(&field.type_annotation),
                        ) {
                            return Err(TypeMismatch(
                                format!(
                                    "Field {} of struct {} must be of type {}",
                                    name.bright_magenta(),
                                    constructor.name.bright_magenta(),
                                    field.type_annotation.type_name.bright_magenta()
                                ),
                                expr.span().clone(),
                            )
                            .into());
                        }
                    } else if !field.type_annotation.is_nullable {
                        return Err(MissingField(
                            name.clone().bright_magenta().to_string(),
                            constructor.name.clone().bright_magenta().to_string(),
                            constructor.token.span.clone(),
                        )
                        .into());
                    }
                }

                Ok(ResolvedType::Struct(
                    constructor.name.clone(),
                    module.id().clone(),
                ))
            }
            Expr::Binary(binary) => {
                let left_type = self.validate_and_get_type_expr(
                    &binary.left,
                    module,
                    ctx,
                    global_type.clone(),
                )?;
                let right_type = self.validate_and_get_type_expr(
                    &binary.right,
                    module,
                    ctx,
                    global_type.clone(),
                )?;

                match binary.operator {
                    _ if binary.operator.is_number_operator() => {
                        match (left_type.clone(), binary.operator, right_type.clone()) {
                            (
                                ResolvedType::String | ResolvedType::Char,
                                BinOpKind::Plus,
                                ResolvedType::String | ResolvedType::Char,
                            ) => Ok(ResolvedType::String),
                            (ResolvedType::Int, _, ResolvedType::Int) => Ok(ResolvedType::Int),
                            (ResolvedType::Float, _, ResolvedType::Float) => {
                                Ok(ResolvedType::Float)
                            }
                            (ResolvedType::Int, _, ResolvedType::Float)
                            | (ResolvedType::Float, _, ResolvedType::Int) => {
                                Ok(ResolvedType::Float)
                            }
                            _ => Err(TypeMismatch(
                                format!(
                                    "Invalid binary operation between {} and {}",
                                    left_type.to_string().bright_magenta(),
                                    right_type.to_string().bright_magenta()
                                ),
                                binary.span().clone(),
                            )
                            .into()),
                        }
                    }
                    _ if binary.operator.is_boolean_operator() => {
                        match (left_type.clone(), binary.operator, right_type.clone()) {
                            (
                                ResolvedType::String | ResolvedType::Char,
                                BinOpKind::Equals,
                                ResolvedType::String | ResolvedType::Char,
                            ) => Ok(ResolvedType::Bool),
                            (ResolvedType::Int, _, ResolvedType::Int) => Ok(ResolvedType::Bool),
                            (ResolvedType::Float, _, ResolvedType::Float) => Ok(ResolvedType::Bool),
                            (ResolvedType::Int, _, ResolvedType::Float)
                            | (ResolvedType::Float, _, ResolvedType::Int) => Ok(ResolvedType::Bool),
                            (ResolvedType::Bool, _, ResolvedType::Bool) => Ok(ResolvedType::Bool),
                            (ResolvedType::Null, _, ResolvedType::Null) => Ok(ResolvedType::Bool),
                            (ResolvedType::Vector(_), _, ResolvedType::Vector(_)) => {
                                Ok(ResolvedType::Bool)
                            }
                            _ => Err(TypeMismatch(
                                format!(
                                    "Invalid boolean operation between {} and {}",
                                    left_type.to_string().bright_magenta(),
                                    right_type.to_string().bright_magenta()
                                ),
                                binary.span().clone(),
                            )
                            .into()),
                        }
                    }
                    _ => {
                        todo!("{:?}", binary)
                    }
                }
            }
            Expr::Assign(assign) => {
                let left_type = self.validate_and_get_type_expr(
                    &assign.left,
                    module,
                    ctx,
                    global_type.clone(),
                )?;
                let right_type = self.validate_and_get_type_expr(
                    &assign.right,
                    module,
                    ctx,
                    global_type.clone(),
                )?;

                match (left_type.clone(), assign.op.clone(), right_type.clone()) {
                    (_, AssignOperator::Assign, _) => {
                        if !ResolvedType::matches(left_type.clone(), right_type.clone()) {
                            return Err(TypeMismatch(
                                format!(
                                    "Cannot assign {} to {}",
                                    right_type.to_string().bright_magenta(),
                                    left_type.to_string().bright_magenta()
                                ),
                                assign.span().clone(),
                            )
                            .into());
                        } else {
                            Ok(left_type.clone())
                        }
                    }
                    (
                        ResolvedType::Int | ResolvedType::Float,
                        AssignOperator::MultiplyEquals
                        | AssignOperator::DivideEquals
                        | AssignOperator::MinusEquals,
                        ResolvedType::Int | ResolvedType::Float,
                    ) => Ok(left_type.clone()),
                    (
                        ResolvedType::Int
                        | ResolvedType::Float
                        | ResolvedType::String
                        | ResolvedType::Char,
                        AssignOperator::PlusEquals,
                        ResolvedType::Int
                        | ResolvedType::Float
                        | ResolvedType::String
                        | ResolvedType::Char,
                    ) => Ok(left_type.clone()),
                    _ => {
                        return Err(TypeMismatch(
                            format!(
                                "Invalid {} assignment between {} and {}",
                                assign.op.to_string().bright_magenta(),
                                left_type.to_string().bright_magenta(),
                                right_type.to_string().bright_magenta()
                            ),
                            assign.span().clone(),
                        )
                        .into());
                    }
                }
            }
            Expr::Variable(var) => {
                if let Some(typ) = self.find_variable(&var.ident) {
                    Ok(typ.clone())
                } else {
                    Err(VariableNotFoundError(var.ident.clone(), var.token.span.clone()).into())
                }
            }
            _ => Ok(ResolvedType::Null),
        }
    }

    pub fn validate_stmt(
        &mut self,
        stmt: &Stmt,
        module: &mut Module,
        ctx: &mut Context,
    ) -> Result<()> {
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

                self.declare_variable(
                    let_stmt.ident.literal().clone(),
                    ResolvedType::from_type_annotation(let_stmt.type_annotation.as_ref().unwrap()),
                );
            }
            _ => {}
        }

        Ok(())
    }

    pub fn validate_block(
        &mut self,
        block: &Vec<Stmt>,
        module: &mut Module,
        ctx: &mut Context,
    ) -> Result<()> {
        self.enter_scope();
        for stmt in block.iter() {
            self.validate_stmt(stmt, module, ctx)?;
        }
        self.exit_scope();

        Ok(())
    }
}
