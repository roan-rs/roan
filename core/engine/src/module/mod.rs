use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};

use anyhow::Result;
use log::debug;
use roan_ast::source::Source;
use roan_ast::{BinOpKind, Expr, Fn, Let, Lexer, Parser, Stmt, Token, Use, Ast, Variable, If, Block};
use roan_error::error::PulseError::{
    ImportError, ModuleNotFoundError, VariableNotFoundError,
};
use roan_error::{print_diagnostic, TextSpan};

use crate::context::Context;
use crate::vm::{Frame, VM};
use crate::vm::value::Value;

pub mod loader;

#[derive(Debug, Clone)]
pub enum ExportType {
    Function(Fn),
    Variable,
}

#[derive(Clone, Debug)]
pub struct Module {
    source: Source,
    path: Option<PathBuf>,
    tokens: Vec<Token>,
    ast: Ast,
    functions: Vec<Fn>,
    exports: Vec<(String, ExportType)>,
    imports: Vec<Use>,
    scopes: Vec<HashMap<String, Value>>, // Stack of scopes
    vm: VM,
}

impl Module {
    /// Creates a new `Module` from the specified `Source`.
    ///
    /// # Parameters
    /// - `source` - The source of the module.
    ///
    /// # Returns
    /// The new `Module`.
    ///
    /// # Examples
    /// ```rust
    /// use roan_engine::module::Module;
    /// use roan_ast::source::Source;
    /// use roan_engine::context::Context;
    /// let source = Source::from_bytes("fn main() { }");
    /// let module = Module::new(source);
    /// ```
    pub fn new(source: Source) -> Self {
        let path = source.path().as_deref().map(Path::to_path_buf);

        Self {
            source,
            path,
            tokens: vec![],
            functions: vec![],
            exports: vec![],
            imports: vec![],
            scopes: vec![HashMap::new()], // Initialize with global scope
            vm: VM::new(),
            ast: Ast::new(),
        }
    }

    /// Returns the path of the module.
    pub fn path(&self) -> Option<PathBuf> {
        self.path.clone()
    }

    /// Returns the source of the module.
    pub fn source(&self) -> &Source {
        &self.source
    }

    /// Parses the module.
    ///
    /// First, the module is lexed into tokens. Then, the tokens are parsed into an AST.
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
        for stmt in self.ast.stmts.clone() {
            self.interpret_stmt(stmt, ctx)?;
        }

        Ok(())
    }

    /// Enter a new scope by pushing a new HashMap onto the scopes stack.
    fn enter_scope(&mut self) {
        debug!("Entering new scope");
        self.scopes.push(HashMap::new());
    }

    /// Exit the current scope by popping the top HashMap from the scopes stack.
    fn exit_scope(&mut self) {
        debug!("Exiting current scope");
        self.scopes.pop();
    }

    /// Declare a new variable in the current (innermost) scope.
    fn declare_variable(&mut self, name: String, val: Value) {
        debug!("Declaring variable '{}' in current scope", name);
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, val);
        }
    }

    /// Set an existing variable's value in the nearest enclosing scope.
    fn set_variable(&mut self, name: &str, val: Value) -> Result<()> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                debug!("Setting variable '{}' to {:?}", name, val);
                scope.insert(name.to_string(), val);
                return Ok(());
            }
        }
        // Variable not found in any scope
        Err(VariableNotFoundError(
            name.to_string(),
            TextSpan::default(),
        )
            .into())
    }

    /// Finds a variable by name, searching from the innermost scope outward.
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

    /// Interpret statement from the module.
    pub fn interpret_stmt(&mut self, stmt: Stmt, ctx: &Context) -> Result<()> {
        match stmt {
            Stmt::Fn(f) => {
                debug!("Interpreting function: {}", f.name);
                self.functions.push(f.clone());

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
                self.interpret_expr(l.initializer.as_ref(), ctx)?;

                let val = self.vm.pop().unwrap();
                let ident = l.ident.literal();
                self.declare_variable(ident.clone(), val);
            }
            Stmt::Expr(expr) => {
                debug!("Interpreting expression: {:?}", expr);

                self.interpret_expr(expr.as_ref(), ctx)?;
            }
            Stmt::Return(r) => {
                debug!("Interpreting return: {:?}", r);

                if let Some(expr) = r.expr {
                    self.interpret_expr(expr.as_ref(), ctx)?;
                }
                // Handle return value as per your VM's design
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
        self.interpret_expr(&if_stmt.condition, ctx)?;
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
                self.interpret_expr(&else_if.condition, ctx)?;
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

    /// Execute a block of statements within a new scope.
    fn execute_block(&mut self, block: Block, ctx: &Context) -> Result<()> {
        self.enter_scope();
        for stmt in block.stmts {
            self.interpret_stmt(stmt, ctx)?;
        }
        self.exit_scope();
        Ok(())
    }

    pub fn interpret_expr(&mut self, expr: &Expr, ctx: &Context) -> Result<()> {
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

                Ok(Value::from_literal(l.clone()))
            }
            Expr::Call(call) => {
                debug!("Interpreting call: {:?}", call);

                let mut args = vec![];

                for arg in call.args.iter() {
                    self.interpret_expr(arg, ctx)?;
                    args.push(
                        self.vm.pop().expect("Expected value on stack"),
                    );
                }

                let function = {
                    let func = self
                        .find_function(&call.callee)
                        .ok_or_else(|| ImportError(call.callee.clone(), call.token.span.clone()))?;
                    func.clone()
                };

                // Enter a new scope for function execution
                self.enter_scope();

                for (param, val) in function.params.iter().zip(args) {
                    let ident = param.ident.literal();
                    self.declare_variable(ident.clone(), val.clone());
                }

                let frame = Frame::new(
                    call.callee.clone(),
                    call.token.span.clone(),
                    Frame::path_or_unknown(self.path()),
                );
                self.vm.push_frame(frame);

                for stmt in function.body.stmts {
                    self.interpret_stmt(stmt, ctx)?;
                }

                self.vm.pop_frame();
                self.exit_scope();

                let val = self.vm.pop()
                    .ok_or_else(|| VariableNotFoundError(call.callee.clone(), call.token.span.clone()))?;
                Ok(val)
            }
            Expr::Parenthesized(p) => {
                debug!("Interpreting parenthesized: {:?}", p);

                self.interpret_expr(&p.expr, ctx)?;

                Ok(self.vm.pop().unwrap())
            }
            Expr::Assign(assign) => {
                debug!("Interpreting assign: {:?}", assign);

                self.interpret_expr(&assign.value, ctx)?;
                let val = self.vm.pop().unwrap();

                let ident = assign.ident.literal();

                self.set_variable(&ident, val.clone())?;

                Ok(val)
            }
            Expr::Binary(b) => {
                debug!("Interpreting binary: {:?}", b);

                self.interpret_expr(&b.left, ctx)?;
                let left = self.vm.pop().unwrap();
                self.interpret_expr(&b.right, ctx)?;
                let right = self.vm.pop().unwrap();

                let val = match (left.clone(), b.operator, right.clone()) {
                    (_, BinOpKind::Plus, _) => left + right,

                    (Value::Int(a), BinOpKind::Minus, Value::Int(b)) => Value::Int(a - b),
                    (Value::Float(a), BinOpKind::Minus, Value::Float(b)) => Value::Float(a - b),
                    (Value::Int(a), BinOpKind::Minus, Value::Float(b)) => Value::Float(a as f64 - b),
                    (Value::Float(a), BinOpKind::Minus, Value::Int(b)) => Value::Float(a - b as f64),

                    (Value::Int(a), BinOpKind::Multiply, Value::Int(b)) => Value::Int(a * b),
                    (Value::Float(a), BinOpKind::Multiply, Value::Float(b)) => Value::Float(a * b),
                    (Value::Int(a), BinOpKind::Multiply, Value::Float(b)) => Value::Float(a as f64 * b),
                    (Value::Float(a), BinOpKind::Multiply, Value::Int(b)) => Value::Float(a * b as f64),

                    (Value::Int(a), BinOpKind::Divide, Value::Int(b)) => Value::Int(a / b),
                    (Value::Float(a), BinOpKind::Divide, Value::Float(b)) => Value::Float(a / b),
                    (Value::Int(a), BinOpKind::Divide, Value::Float(b)) => Value::Float(a as f64 / b),
                    (Value::Float(a), BinOpKind::Divide, Value::Int(b)) => Value::Float(a / b as f64),

                    (Value::Int(a), BinOpKind::Equals, Value::Int(b)) => Value::Bool(a == b),
                    (Value::Float(a), BinOpKind::Equals, Value::Float(b)) => Value::Bool(a == b),
                    (Value::String(a), BinOpKind::Equals, Value::String(b)) => Value::Bool(a == b),

                    (Value::Int(a), BinOpKind::BangEquals, Value::Int(b)) => Value::Bool(a != b),
                    (Value::Float(a), BinOpKind::BangEquals, Value::Float(b)) => Value::Bool(a != b),
                    (Value::String(a), BinOpKind::BangEquals, Value::String(b)) => Value::Bool(a != b),

                    (Value::Bool(a), BinOpKind::And, Value::Bool(b)) => Value::Bool(a && b),
                    (Value::Bool(a), BinOpKind::Or, Value::Bool(b)) => Value::Bool(a || b),

                    (Value::Int(a), BinOpKind::GreaterThan, Value::Int(b)) => Value::Bool(a > b),
                    (Value::Float(a), BinOpKind::GreaterThan, Value::Float(b)) => Value::Bool(a > b),
                    (Value::Int(a), BinOpKind::GreaterThan, Value::Float(b)) => Value::Bool(a as f64 > b),
                    (Value::Float(a), BinOpKind::GreaterThan, Value::Int(b)) => Value::Bool(a > b as f64),

                    _ => todo!("missing binary operator: {:?}", b.operator),
                };

                Ok(val)
            }

            _ => todo!("missing expr: {:?}", expr),
        };

        self.vm.push(val?);

        Ok(())
    }

    /// Looks for a function with the specified name.
    pub fn find_function(&self, name: &str) -> Option<&Fn> {
        debug!("Looking for function: {}", name);
        self.functions.iter().find(|f| f.name == name)
    }
}
