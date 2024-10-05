use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};

use anyhow::Result;
use log::debug;
use roan_ast::source::Source;
use roan_ast::{BinOpKind, Expr, Fn, Lexer, Parser, Stmt, Token, Use, Ast, If, Block};
use roan_error::error::PulseError::{ImportError, ModuleNotFoundError, UndefinedFunctionError, VariableNotFoundError};
use roan_error::{print_diagnostic, TextSpan};
use uuid::Uuid;
use crate::context::Context;
use crate::module::loader::remove_surrounding_quotes;
use crate::natives::get_stored_function;
use crate::natives::io::__print;
use crate::vm::{Frame, VM};
use crate::vm::native_fn::NativeFunction;
use crate::vm::value::Value;

pub mod loader;

#[derive(Debug, Clone)]
pub enum ExportType {
    Function(Fn),
    Variable,
}

#[derive(Debug, Clone)]
pub enum StoredFunction {
    Native(NativeFunction),
    Function(Fn),
}

#[derive(Clone, Debug)]
pub struct Module {
    source: Source,
    path: Option<PathBuf>,
    tokens: Vec<Token>,
    pub(crate) ast: Ast,
    functions: Vec<StoredFunction>,
    exports: Vec<(String, ExportType)>,
    imports: Vec<Use>,
    scopes: Vec<HashMap<String, Value>>, // Stack of scopes
    vm: VM,
}

impl Module {
    pub fn new(source: Source) -> Self {
        let path = source.path().as_deref().map(Path::to_path_buf);

        Self {
            source,
            path,
            tokens: vec![],
            functions: get_stored_function(),
            exports: vec![],
            imports: vec![],
            scopes: vec![HashMap::new()], // Initialize with global scope
            vm: VM::new(),
            ast: Ast::new(),
        }
    }

    pub fn path(&self) -> Option<PathBuf> {
        self.path.clone()
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    pub fn parse(&mut self) -> Result<()> {
        debug!("Parsing module from source");
        let mut lexer = Lexer::new(self.source.clone());

        let tokens = lexer.lex()?;
        debug!("Parsed {} tokens", tokens.len());
        self.tokens = tokens;

        let mut parser = Parser::new(self.tokens.clone());

        debug!("Parsing tokens into AST");

        let ast = parser.parse()?;

        self.ast = ast;

        Ok(())
    }

    pub fn interpret(&mut self, ctx: &Context) -> Result<()> {
        println!("{:#?}", self.ast);
        // for stmt in self.ast.stmts.clone() {
        //     match self.interpret_stmt(stmt.0, ctx) {
        //         Ok(_) => {}
        //         Err(e) => {
        //             print_diagnostic(e, Some(self.source.content()));
        // 
        //             // Is this a good idea?
        //             std::process::exit(1);
        //         }
        //     }
        // }

        Ok(())
    }

    fn enter_scope(&mut self) {
        debug!("Entering new scope");
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        debug!("Exiting current scope");
        self.scopes.pop();
    }

    fn declare_variable(&mut self, name: String, val: Value) {
        debug!("Declaring variable '{}' in current scope", name);
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, val);
        }
    }

    fn set_variable(&mut self, name: &str, val: Value) -> Result<()> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                debug!("Setting variable '{}' to {:?}", name, val);
                scope.insert(name.to_string(), val);
                return Ok(());
            }
        }
        Err(VariableNotFoundError(
            name.to_string(),
            TextSpan::default(),
        )
            .into())
    }

    pub fn find_variable(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                debug!("Found variable '{}' with value {:?}", name, val);
                return Some(val);
            }
        }
        debug!("Variable '{}' not found in any scope", name);
        None
    }

    pub fn name(&self) -> String {
        self.path().unwrap().file_stem().unwrap().to_string_lossy().to_string()
    }

    pub fn interpret_stmt(&mut self, stmt: Uuid, ctx: &Context) -> Result<()> {
        let stmt = self.ast.query(stmt).clone();

        match stmt {
            Stmt::Fn(f) => {
                debug!("Interpreting function: {}", f.name);
                self.functions.push(StoredFunction::Function(f.clone()));

                if f.exported {
                    self.exports
                        .push((f.name.clone(), ExportType::Function(f.clone())));
                }
            }
            Stmt::Use(u) => {
                debug!("Interpreting use: {}", u.from.literal());
                let mut module = ctx
                    .module_loader
                    .load(&self, &u.from.literal(), ctx)
                    .map_err(|_| ModuleNotFoundError(u.from.literal(), u.from.span.clone()))?;

                if let Err(e) = module.parse() {
                    print_diagnostic(e, Some(module.source().content()));
                    return Err(anyhow::anyhow!("Failed to parse module"));
                }

                module.interpret(ctx)?;

                let imported_items: Vec<(String, &Token)> = u
                    .items
                    .iter()
                    .map(|i| (i.literal(), i))
                    .collect();

                for (name, item) in imported_items {
                    match module.find_function(&name) {
                        Some(f) => {
                            self.functions.push(f.clone());
                        }
                        None => {
                            return Err(ImportError(name, item.span.clone()).into());
                        }
                    }
                }
            }
            Stmt::Let(l) => {
                debug!("Interpreting let: {:?}", l.ident);
                let initializer_clone = l.initializer.clone();
                self.interpret_expr(initializer_clone, ctx)?;

                let val = self.vm.pop().unwrap();
                let ident = l.ident.literal();
                self.declare_variable(ident.clone(), val);
            }
            Stmt::Expr(expr) => {
                debug!("Interpreting expression: {:?}", expr);

                let expr_clone = expr.clone();
                self.interpret_expr(expr_clone.clone(), ctx)?;
            }
            Stmt::Return(r) => {
                debug!("Interpreting return: {:?}", r);

                if let Some(expr) = r.expr {
                    let expr_clone = expr.clone();
                    self.interpret_expr(expr_clone, ctx)?;
                }
            }
            Stmt::Block(block) => {
                debug!("Interpreting block statement");
                self.execute_block(block, ctx)?;
            }
            Stmt::If(if_stmt) => {
                debug!("Interpreting if statement");
                self.interpret_if(if_stmt, ctx)?;
            }
        }

        Ok(())
    }

    fn interpret_if(&mut self, if_stmt: If, ctx: &Context) -> Result<()> {
        let condition_clone = if_stmt.condition.clone();
        self.interpret_expr(condition_clone, ctx)?;
        let condition_value = self.vm.pop().ok_or_else(|| {
            anyhow::anyhow!("Expected a value on the VM stack for if condition")
        })?;

        let condition = match condition_value {
            Value::Bool(b) => b,
            _ => {
                return Err(anyhow::anyhow!(
                    "If condition does not evaluate to a boolean"
                ))
            }
        };

        if condition {
            self.execute_block(if_stmt.then_block, ctx)?;
        } else {
            let mut executed = false;
            for else_if in if_stmt.else_ifs {
                let else_if_condition_clone = else_if.condition.clone();
                self.interpret_expr(else_if_condition_clone, ctx)?;
                let else_if_condition = self.vm.pop().ok_or_else(|| {
                    anyhow::anyhow!(
                        "Expected a value on the VM stack for else-if condition"
                    )
                })?;

                let else_if_result = match else_if_condition {
                    Value::Bool(b) => b,
                    _ => {
                        return Err(anyhow::anyhow!(
                            "Else-if condition does not evaluate to a boolean"
                        ))
                    }
                };

                if else_if_result {
                    self.execute_block(else_if.block, ctx)?;
                    executed = true;
                    break;
                }
            }

            if !executed {
                if let Some(else_block) = if_stmt.else_block {
                    self.execute_block(else_block.block, ctx)?;
                }
            }
        }

        Ok(())
    }

    fn execute_block(&mut self, block: Block, ctx: &Context) -> Result<()> {
        self.enter_scope();
        for stmt in block.stmts {
            self.interpret_stmt(stmt.clone(), ctx)?;
        }
        self.exit_scope();
        Ok(())
    }

    pub fn interpret_expr(&mut self, expr: Uuid, ctx: &Context) -> Result<()> {
        let expr = self.ast.query_expr(expr).clone();

        let val: Result<Value> = match expr {
            Expr::Variable(v) => {
                debug!("Interpreting variable: {}", v.ident);

                let variable = self
                    .find_variable(&v.ident)
                    .ok_or_else(|| VariableNotFoundError(
                        v.ident.clone(),
                        v.token.span.clone(),
                    ))?;

                Ok(variable.clone())
            }
            Expr::Literal(l) => {
                debug!("Interpreting literal: {:?}", l);
                Ok(Value::from_literal(l))
            }
            Expr::Call(call) => {
                debug!("Interpreting call: {:?}", call);

                let mut args = vec![];

                let call_args = call.args.clone();
                for arg in call_args.iter() {
                    self.interpret_expr(*arg, ctx)?;
                    args.push(
                        self.vm.pop().expect("Expected value on stack"),
                    );
                }

                let function = {
                    let func = self
                        .find_function(&call.callee)
                        .ok_or_else(|| UndefinedFunctionError(call.callee.clone(), call.token.span.clone()))?;
                    func.clone()
                };

                self.enter_scope();

                match function {
                    StoredFunction::Native(n) => {
                        let mut params = vec![];
                        for (param, val) in n.params.iter().zip(args.clone()) {
                            if param.is_rest {
                                let rest = args.iter().skip(n.params.len() - 1).cloned().collect();
                                params.push(Value::Vec(rest));
                            } else {
                                params.push(val);
                            }
                        }

                        let result = (n.func)(params);
                        self.vm.push(result);
                    }
                    StoredFunction::Function(f) => {
                        for (param, val) in f.params.iter().zip(args.clone()) {
                            let ident = param.ident.literal();

                            if param.is_rest {
                                let rest = args.iter().skip(f.params.len() - 1).cloned().collect();
                                self.declare_variable(ident, Value::Vec(rest));
                            } else {
                                self.declare_variable(ident, val);
                            }
                        }

                        let frame = Frame::new(
                            call.callee.clone(),
                            call.token.span.clone(),
                            Frame::path_or_unknown(self.path()),
                        );
                        self.vm.push_frame(frame);

                        let stmts = f.body.stmts.clone();
                        for stmt_id in stmts {
                            self.interpret_stmt(stmt_id, ctx)?;
                        }
                    }
                };

                self.vm.pop_frame();
                self.exit_scope();

                let val = self.vm.pop().or(Some(Value::Void)).unwrap();

                Ok(val)
            }
            Expr::Parenthesized(p) => {
                debug!("Interpreting parenthesized: {:?}", p);

                self.interpret_expr(p.expr, ctx)?;

                Ok(self.vm.pop().unwrap())
            }
            Expr::Assign(assign) => {
                debug!("Interpreting assign: {:?}", assign);

                self.interpret_expr(assign.value, ctx)?;
                let val = self.vm.pop().unwrap();

                let ident = assign.ident.literal();

                self.set_variable(&ident, val.clone())?;

                Ok(val)
            }
            Expr::Vec(vec) => {
                debug!("Interpreting vec: {:?}", vec);

                let mut values = vec![];
                let vec_exprs = vec.exprs.clone();

                for expr_id in vec_exprs.iter() {
                    self.interpret_expr(*expr_id, ctx)?;
                    values.push(self.vm.pop().unwrap());
                }

                Ok(Value::Vec(values))
            }
            Expr::Binary(b) => {
                debug!("Interpreting binary: {:?}", b);

                let b_left_clone = b.left.clone();
                let b_operator = b.operator.clone();
                let b_right_clone = b.right.clone();

                self.interpret_expr(b_left_clone, ctx)?;
                let left = self.vm.pop().unwrap();

                self.interpret_expr(b_right_clone, ctx)?;
                let right = self.vm.pop().unwrap();

                let val = match (left.clone(), b_operator, right.clone()) {
                    (_, BinOpKind::Plus, _) => left + right,
                    (_, BinOpKind::Minus, _) => left - right,
                    (_, BinOpKind::Multiply, _) => left * right,
                    (_, BinOpKind::Divide, _) => left / right,
                    (_, BinOpKind::Equals, _) => Value::Bool(left == right),
                    (_, BinOpKind::BangEquals, _) => Value::Bool(left != right),
                    (_, BinOpKind::GreaterThan, _) => Value::Bool(left > right),
                    (_, BinOpKind::LessThan, _) => Value::Bool(left < right),
                    (_, BinOpKind::GreaterThanOrEqual, _) => Value::Bool(left >= right),
                    (_, BinOpKind::LessThanOrEqual, _) => Value::Bool(left <= right),

                    (Value::Bool(a), BinOpKind::And, Value::Bool(b)) => Value::Bool(a && b),
                    (Value::Bool(a), BinOpKind::Or, Value::Bool(b)) => Value::Bool(a || b),

                    (Value::Int(a), BinOpKind::Modulo, Value::Int(b)) => Value::Int(a % b),
                    (Value::Float(a), BinOpKind::Modulo, Value::Float(b)) => Value::Float(a % b),
                    (Value::Int(a), BinOpKind::Modulo, Value::Float(b)) => Value::Float(a as f64 % b),
                    (Value::Float(a), BinOpKind::Modulo, Value::Int(b)) => Value::Float(a % b as f64),

                    (Value::Int(a), BinOpKind::And, Value::Int(b)) => Value::Int(a & b),
                    (Value::Int(a), BinOpKind::Or, Value::Int(b)) => Value::Int(a | b),
                    (Value::Int(a), BinOpKind::BitwiseXor, Value::Int(b)) => Value::Int(a ^ b),

                    (Value::Int(a), BinOpKind::Power, Value::Int(b)) => Value::Int(a.pow(b as u32)),
                    (Value::Float(a), BinOpKind::Power, Value::Float(b)) => Value::Float(a.powf(b)),
                    (Value::Int(a), BinOpKind::Power, Value::Float(b)) => Value::Float((a as f64).powf(b)),
                    (Value::Float(a), BinOpKind::Power, Value::Int(b)) => Value::Float(a.powi(b as i32)),

                    _ => todo!("missing binary operator: {:?}", b.operator),
                };

                Ok(val)
            }

            _ => todo!("missing expr: {:?}", expr),
        };

        self.vm.push(val?);

        Ok(())
    }

    pub fn find_function(&self, name: &str) -> Option<&StoredFunction> {
        debug!("Looking for function: {}", name);

        self.functions.iter().find(|f| {
            match f {
                StoredFunction::Native(n) => n.name == name,
                StoredFunction::Function(f) => f.name == name,
            }
        })
    }
}