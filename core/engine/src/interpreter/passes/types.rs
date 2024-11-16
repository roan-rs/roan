use crate::{
    context::Context,
    interpreter::passes::Pass,
    module::{Module, StoredFunction},
    value::Value,
    vm::{
        native_fn::{NativeFunction, NativeFunctionParam},
        VM,
    },
};
use anyhow::Result;
use colored::Colorize;
use indexmap::IndexMap;
use roan_ast::{
    AccessKind, AssignOperator, BinOpKind, Expr, GetSpan, LiteralType, Stmt, TypeAnnotation,
    UnOpKind,
};
use roan_error::{
    error::RoanError::{
        MissingField, MissingParameter, PropertyNotFoundError, StaticContext, StaticMemberAccess,
        TypeMismatch, UndefinedFunctionError, VariableNotFoundError,
    },
    TextSpan,
};
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

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
        _: &mut VM,
    ) -> Result<()> {
        self.validate_stmt(&stmt, module, ctx)?;

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
    Void,
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
            ResolvedType::Void => write!(f, "void"),
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
            (ResolvedType::Void, ResolvedType::Void) => true,
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
                ResolvedType::Object(_) => "object".to_string(),
                ResolvedType::Vector(_) => "vec".to_string(),
                ResolvedType::Any => "anytype".to_string(),
                ResolvedType::Void => "void".to_string(),
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
            "void" => ResolvedType::Void,
            _ => {
                // Not sure if this is the best approach
                if let Some(mod_id) = &typ.module_id {
                    ResolvedType::Struct(typ.type_name.clone(), mod_id.clone())
                } else {
                    ResolvedType::Any
                }
            }
        }
    }

    pub fn matches_to(type1: ResolvedType, type2: ResolvedType, to: ResolvedType) -> bool {
        ResolvedType::matches(type1, to.clone()) && ResolvedType::matches(type2, to)
    }

    pub fn built_in(&self) -> HashMap<String, NativeFunction> {
        let value = match self {
            ResolvedType::Int => Value::Int(0),
            ResolvedType::Float => Value::Float(0.0),
            ResolvedType::Bool => Value::Bool(false),
            ResolvedType::String => Value::String("".to_string()),
            ResolvedType::Char => Value::Char(' '),
            ResolvedType::Null => Value::Null,
            ResolvedType::Object(_) => Value::Object(IndexMap::new()),
            ResolvedType::Vector(_) => Value::Vec(vec![]),
            ResolvedType::Void => Value::Null,
            _ => Value::Null,
        };

        value.builtin_methods()
    }

    pub fn from_value(value: Value, mod_id: String) -> ResolvedType {
        match value {
            Value::Int(_) => ResolvedType::Int,
            Value::Float(_) => ResolvedType::Float,
            Value::Bool(_) => ResolvedType::Bool,
            Value::String(_) => ResolvedType::String,
            Value::Char(_) => ResolvedType::Char,
            Value::Null => ResolvedType::Null,
            Value::Object(fields) => ResolvedType::Object(Box::new(
                fields
                    .iter()
                    .map(|(_, v)| ResolvedType::from_value(v.clone(), mod_id.clone()))
                    .next()
                    .unwrap_or(ResolvedType::Any),
            )),
            Value::Vec(items) => ResolvedType::Vector(Box::new(ResolvedType::from_value(
                items
                    .iter()
                    .map(|v| v.clone())
                    .next()
                    .unwrap_or(Value::Null),
                mod_id,
            ))),
            Value::Struct(name, _) => ResolvedType::Struct(name.name.literal(), mod_id),
            Value::Void => ResolvedType::Void,
        }
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

        for stmt in &func.body.stmts {
            self.validate_stmt(stmt, module, ctx)?;
        }

        Ok(())
    }

    pub fn annotation_from_native_param(param: NativeFunctionParam) -> TypeAnnotation {
        TypeAnnotation {
            separator: None,
            token_name: None,
            type_name: param.ty,
            is_array: false,
            is_nullable: false,
            module_id: None,
            is_generic: false,
            generics: vec![],
        }
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
                for (_, value) in &obj.fields {
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
            Expr::Parenthesized(expr) => {
                self.validate_and_get_type_expr(&expr.expr, module, ctx, global_type)
            }
            Expr::StructConstructor(constructor) => {
                let struct_type =
                    module.get_struct(&constructor.name, constructor.token.span.clone())?;

                for (name, field) in &struct_type.fields {
                    let constructor_field = constructor.fields.iter().find(|(n, _)| n == &name);

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
                        ResolvedType::Int | ResolvedType::Float | ResolvedType::Any,
                        AssignOperator::MultiplyEquals
                        | AssignOperator::DivideEquals
                        | AssignOperator::MinusEquals,
                        ResolvedType::Int | ResolvedType::Float | ResolvedType::Any,
                    ) => Ok(left_type.clone()),
                    (
                        ResolvedType::Int
                        | ResolvedType::Float
                        | ResolvedType::String
                        | ResolvedType::Char
                        | ResolvedType::Any,
                        AssignOperator::PlusEquals,
                        ResolvedType::Int
                        | ResolvedType::Float
                        | ResolvedType::String
                        | ResolvedType::Char
                        | ResolvedType::Any,
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
                    if let Some(cnst) = module.find_const(&var.ident) {
                        Ok(ResolvedType::from_value(
                            cnst.value.clone(),
                            cnst.defining_module.clone(),
                        ))
                    } else {
                        Err(VariableNotFoundError(var.ident.clone(), var.token.span.clone()).into())
                    }
                }
            }
            Expr::Call(call) => {
                let stored_function = module
                    .find_function(&call.callee)
                    .ok_or_else(|| {
                        UndefinedFunctionError(call.callee.clone(), call.token.span.clone())
                    })?
                    .clone();

                let mut arg_types = vec![];

                for arg in &call.args {
                    arg_types.push(self.validate_and_get_type_expr(
                        arg,
                        module,
                        ctx,
                        global_type.clone(),
                    )?);
                }

                let mut param_types: Vec<(ResolvedType, bool, bool)> = vec![];
                let mut typ: Option<TypeAnnotation> = None;

                match stored_function {
                    StoredFunction::Native(native) => {
                        for param in &native.params {
                            param_types.push((
                                ResolvedType::from_type_annotation(
                                    &Self::annotation_from_native_param(param.clone()),
                                ),
                                true,
                                param.is_rest,
                            ));
                        }
                        typ = Some(TypeAnnotation {
                            separator: None,
                            token_name: None,
                            type_name: "anytype".to_string(),
                            is_array: false,
                            is_nullable: true,
                            module_id: None,
                            is_generic: false,
                            generics: vec![],
                        });
                    }
                    StoredFunction::Function { function, .. } => {
                        for param in &function.params {
                            param_types.push((
                                ResolvedType::from_type_annotation(&param.type_annotation),
                                param.type_annotation.is_nullable,
                                param.is_rest,
                            ));
                        }
                        typ = function.return_type.clone();
                    }
                }

                let mut arg_index = 0;

                for (param_type, nullable, is_rest) in param_types.iter() {
                    if *is_rest {
                        // Handle rest parameter: All remaining arguments must match the `param_type`
                        while let Some(arg_type) = arg_types.get(arg_index) {
                            if !ResolvedType::matches(param_type.clone(), arg_type.clone()) {
                                return Err(TypeMismatch(
                                    format!(
                                        "Expected type {} for rest arguments but got {}",
                                        param_type.to_string().bright_magenta(),
                                        arg_type.to_string().bright_magenta()
                                    ),
                                    call.args[arg_index].span().clone(),
                                )
                                .into());
                            }
                            arg_index += 1;
                        }
                        break; // All remaining arguments processed as rest
                    } else {
                        // Non-rest parameter
                        if let Some(arg_type) = arg_types.get(arg_index) {
                            if !ResolvedType::matches(param_type.clone(), arg_type.clone()) {
                                return Err(TypeMismatch(
                                    format!(
                                        "Expected type {} but got {}",
                                        param_type.to_string().bright_magenta(),
                                        arg_type.to_string().bright_magenta()
                                    ),
                                    call.args[arg_index].span().clone(),
                                )
                                .into());
                            }
                            arg_index += 1;
                        } else if !nullable {
                            return Err(MissingParameter(
                                call.callee.clone().bright_magenta().to_string(),
                                call.token.span.clone(),
                            )
                            .into());
                        }
                    }
                }
                let typ = &mut typ.unwrap_or_else(|| TypeAnnotation {
                    separator: None,
                    token_name: None,
                    type_name: "void".to_string(),
                    is_array: false,
                    is_nullable: true,
                    module_id: None,
                    is_generic: false,
                    generics: vec![],
                });

                typ.module_id = Some(module.id().clone());

                Ok(ResolvedType::from_type_annotation(typ))
            }
            Expr::Access(access) => match access.access.clone() {
                AccessKind::Index(expr) => {
                    let base = self.validate_and_get_type_expr(
                        access.base.as_ref(),
                        module,
                        ctx,
                        global_type.clone(),
                    )?;

                    let index_type =
                        self.validate_and_get_type_expr(&expr, module, ctx, global_type.clone())?;

                    match (base.clone(), index_type.clone()) {
                        (ResolvedType::Vector(t), ResolvedType::Int) => Ok(*t),
                        (ResolvedType::String, ResolvedType::Int) => Ok(ResolvedType::Char),
                        (ResolvedType::Object(t), ResolvedType::String) => Ok(*t),
                        (ResolvedType::Object(_), _) => Err(TypeMismatch(
                            "Objects can only be indexed with strings".to_string(),
                            expr.span().clone(),
                        )
                        .into()),
                        (ResolvedType::Vector(_), _) => Err(TypeMismatch(
                            "Vectors can only be indexed with integers".to_string(),
                            expr.span().clone(),
                        )
                        .into()),
                        (ResolvedType::String, _) => Err(TypeMismatch(
                            "Strings can only be indexed with integers".to_string(),
                            expr.span().clone(),
                        )
                        .into()),
                        (ResolvedType::Any, _) => Ok(ResolvedType::Any),
                        _ => Err(TypeMismatch(
                            format!(
                                "Invalid index operation. Attempted to access {} with {}",
                                base.to_string().bright_magenta(),
                                index_type.to_string().bright_magenta()
                            ),
                            expr.span().clone(),
                        )
                        .into()),
                    }
                }
                AccessKind::Field(expr) => {
                    let base = self.validate_and_get_type_expr(
                        access.base.as_ref(),
                        module,
                        ctx,
                        global_type.clone(),
                    )?;

                    match expr.as_ref().clone() {
                        Expr::Call(call) => {
                            match base {
                                ResolvedType::Struct(name, id) => {
                                    let module = ctx.query_module(&id).unwrap();
                                    let struct_def = module.get_struct(&name, expr.span())?;

                                    let field = struct_def.find_method(&call.callee);

                                    if field.is_none() {
                                        return Err(PropertyNotFoundError(
                                            call.callee.clone(),
                                            call.token.span.clone(),
                                        )
                                        .into());
                                    }

                                    Ok(ResolvedType::from_type_annotation(
                                        &field.unwrap().return_type.clone().unwrap_or(
                                            TypeAnnotation {
                                                separator: None,
                                                token_name: None,
                                                type_name: "void".to_string(),
                                                is_array: false,
                                                is_nullable: true,
                                                module_id: None,

                                                is_generic: false,
                                                generics: vec![],
                                            },
                                        ),
                                    ))
                                }
                                _ => {
                                    if let Some(_) = base.built_in().get(&call.callee) {
                                        Ok(ResolvedType::from_type_annotation(&TypeAnnotation {
                                            separator: None,
                                            token_name: None,
                                            type_name: "anytype".to_string(),
                                            is_array: false,
                                            is_nullable: true,
                                            module_id: None,
                                            is_generic: false,
                                            generics: vec![],
                                        }))
                                    } else {
                                        Err(PropertyNotFoundError(
                                            call.callee.clone(),
                                            call.token.span.clone(),
                                        )
                                        .into())
                                    }
                                }
                            }
                        }
                        Expr::Variable(lit) => match base {
                            // We could possibly check if the field exists in the object here
                            ResolvedType::Object(typ) => Ok(*typ),
                            ResolvedType::Struct(name, id) => {
                                let module = ctx.query_module(&id).unwrap();
                                let struct_def = module.get_struct(&name, expr.span())?;

                                let field = struct_def.find_field(&lit.ident);

                                if field.is_none() {
                                    return Err(PropertyNotFoundError(
                                        lit.ident.clone(),
                                        lit.token.span.clone(),
                                    )
                                    .into());
                                }

                                Ok(ResolvedType::from_type_annotation(
                                    &field.unwrap().type_annotation,
                                ))
                            }
                            _ => Err(TypeMismatch(
                                format!(
                                    "Cannot access field of {} type",
                                    base.to_string().bright_magenta().to_string()
                                ),
                                lit.token.span.clone(),
                            )
                            .into()),
                        },
                        _ => Err(TypeMismatch(
                            "Invalid field access".to_string(),
                            expr.span().clone(),
                        )
                        .into()),
                    }
                }
                AccessKind::StaticMethod(expr) => {
                    let base = access.base.as_ref().clone();
                    let (struct_name, span) = match base {
                        Expr::Variable(v) => (v.ident.clone(), v.token.span.clone()),
                        _ => return Err(StaticMemberAccess(access.span()).into()),
                    };

                    let struct_def = module.get_struct(&struct_name, span)?;

                    match expr.as_ref().clone() {
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

                            let function = method.unwrap().clone();
                            module.functions.push(StoredFunction::Function {
                                function: function.clone(),
                                defining_module: module.id().clone(),
                            });

                            let typ =
                                self.validate_and_get_type_expr(expr.as_ref(), module, ctx, None)?;

                            module.functions.remove(
                                module.functions.iter().position(|f| {
                                    matches!(f, StoredFunction::Function { function, .. } if function.name == method_name)
                                }).unwrap()
                            );

                            Ok(typ)
                        }
                        _ => Err(StaticContext(expr.span()).into()),
                    }
                }
            },
            _ => Ok(ResolvedType::Null),
        }
    }

    pub fn validate_stmt(
        &mut self,
        stmt: &Stmt,
        module: &mut Module,
        ctx: &mut Context,
    ) -> Result<()> {
        match stmt.clone() {
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

                    let_stmt.type_annotation = Some(typ_clone);
                }

                self.declare_variable(
                    let_stmt.ident.literal().clone(),
                    ResolvedType::from_type_annotation(let_stmt.type_annotation.as_ref().unwrap()),
                );
            }
            Stmt::Fn(mut func) => {
                self.enter_scope();
                for param in func.params.iter_mut() {
                    self.check_type_annotation(&mut param.type_annotation, module, ctx)?;

                    self.declare_variable(
                        param.ident.clone().literal(),
                        ResolvedType::from_type_annotation(&param.type_annotation),
                    );
                }
                self.validate_function(&mut func, module, ctx)?;
                self.exit_scope()
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
