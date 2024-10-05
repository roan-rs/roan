use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use anyhow::Result;
use log::debug;
use roan_ast::source::Source;
use roan_ast::{BinOpKind, Expr, Fn, Lexer, Parser, Stmt, Token, Use, Ast, If, Block};
use roan_error::error::PulseError::{ImportError, ModuleNotFoundError, UndefinedFunctionError, VariableNotFoundError};
use roan_error::{print_diagnostic, TextSpan};

use crate::context::Context;
use crate::natives::get_stored_function;
use crate::vm::{Frame, VM};
use crate::vm::native_fn::NativeFunction;
use crate::vm::value::Value;

pub mod loader;

#[derive(Debug, Clone)]
pub enum ExportType {
    Function(Fn),
    Variable,
}

/// Represents a function stored in a module.
#[derive(Debug, Clone)]
pub enum StoredFunction {
    Native(NativeFunction),
    Function {
        function: Fn,
        defining_module: Arc<Mutex<Module>>,
    },
}

#[derive(Clone, Debug)]
pub struct Module {
    source: Source,
    path: Option<PathBuf>,
    tokens: Vec<Token>,
    ast: Ast,
    functions: Vec<StoredFunction>,
    exports: Vec<(String, ExportType)>,
    imports: Vec<Use>,
    scopes: Vec<HashMap<String, Value>>, // Stack of scopes
    vm: VM,
}

impl Module {
    /// Creates a new Module from the specified Source.
    ///
    /// # Parameters
    /// - source - The source of the module.
    ///
    /// # Returns
    /// An `Arc<Mutex<Self>>` containing the new Module.
    pub fn new(source: Source) -> Arc<Mutex<Self>> {
        let path = source.path().as_deref().map(Path::to_path_buf);

        let module = Self {
            source,
            path,
            tokens: vec![],
            functions: get_stored_function(),
            exports: vec![],
            imports: vec![],
            scopes: vec![HashMap::new()], // Initialize with global scope
            vm: VM::new(),
            ast: Ast::new(),
        };

        Arc::new(Mutex::new(module))
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
            match self.interpret_stmt(stmt, ctx) {
                Ok(_) => {}
                Err(e) => {
                    print_diagnostic(e, Some(self.source.content()));
                    return Err(anyhow::anyhow!("Failed to interpret module"));
                }
            }
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

    pub fn name(&self) -> String {
        self.path().unwrap().file_stem().unwrap().to_string_lossy().to_string()
    }

    /// Interpret statement from the module.
    pub fn interpret_stmt(&mut self, stmt: Stmt, ctx: &Context) -> Result<()> {
        match stmt {
            Stmt::Fn(f) => {
                debug!("Interpreting function: {}", f.name);
                self.functions.push(StoredFunction::Function {
                    function: f.clone(),
                    defining_module: Arc::clone(&Arc::new(Mutex::new(self.clone()))),
                });

                if f.exported {
                    self.exports
                        .push((f.name.clone(), ExportType::Function(f.clone())));
                }
            }
            Stmt::Use(u) => {
                debug!("Interpreting use: {}", u.from.literal());

                // Load the module as an Arc<Mutex<Module>>
                let module = ctx
                    .module_loader
                    .load(&self.clone(), &u.from.literal(), ctx)
                    .map_err(|_| ModuleNotFoundError(u.from.literal().to_string(), u.from.span.clone()))?;

                // Lock the loaded module for parsing and interpretation
                let mut loaded_module = module.lock().expect("Failed to lock loaded module");

                match loaded_module.parse() {
                    Ok(_) => {}
                    Err(e) => {
                        print_diagnostic(e, Some(loaded_module.source().content()));
                        return Err(anyhow::anyhow!("Failed to parse module"));
                    }
                }

                match loaded_module.interpret(ctx) {
                    Ok(_) => {}
                    Err(e) => {
                        print_diagnostic(e, Some(loaded_module.source().content()));
                        return Err(anyhow::anyhow!("Failed to interpret module"));
                    }
                }

                // Collect the items to import
                let imported_items: Vec<(String, &Token)> = u
                    .items
                    .iter()
                    .map(|i| (i.literal(), i))
                    .collect();

                for (name, item) in imported_items {
                    match loaded_module.find_function(&name) {
                        Some(StoredFunction::Function { function, defining_module }) => {
                            self.functions.push(StoredFunction::Function {
                                function: function.clone(),
                                defining_module: Arc::clone(&defining_module),
                            });
                        }
                        Some(StoredFunction::Native(n)) => {
                            self.functions.push(StoredFunction::Native(n.clone()));
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
                    args.push(self.vm.pop().expect("Expected value on stack"));
                }

                let stored_function = self
                    .find_function(&call.callee)
                    .ok_or_else(|| UndefinedFunctionError(call.callee.clone(), call.token.span.clone()))?
                    .clone();

                match stored_function {
                    StoredFunction::Native(n) => {
                        self.execute_native_function(n, args)?;
                    }
                    StoredFunction::Function { function, defining_module } => {
                        self.execute_user_defined_function(function, defining_module, args, ctx)?;
                    }
                }

                Ok(self.vm.pop().unwrap())
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
            Expr::Vec(vec) => {
                debug!("Interpreting vec: {:?}", vec);

                let mut values = vec![];

                for expr in vec.exprs.iter() {
                    self.interpret_expr(expr, ctx)?;
                    values.push(self.vm.pop().unwrap());
                }

                Ok(Value::Vec(values))
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

                    (Value::Int(a), BinOpKind::LessThan, Value::Int(b)) => Value::Bool((a as f64) < (b as f64)),
                    (Value::Float(a), BinOpKind::LessThan, Value::Float(b)) => Value::Bool(a < b),
                    (Value::Int(a), BinOpKind::LessThan, Value::Float(b)) => Value::Bool((a as f64) < b),
                    (Value::Float(a), BinOpKind::LessThan, Value::Int(b)) => Value::Bool(a < (b as f64)),

                    (Value::Int(a), BinOpKind::GreaterThanOrEqual, Value::Int(b)) => Value::Bool(a >= b),
                    (Value::Float(a), BinOpKind::GreaterThanOrEqual, Value::Float(b)) => Value::Bool(a >= b),
                    (Value::Int(a), BinOpKind::GreaterThanOrEqual, Value::Float(b)) => Value::Bool(a as f64 >= b),
                    (Value::Float(a), BinOpKind::GreaterThanOrEqual, Value::Int(b)) => Value::Bool(a >= b as f64),

                    (Value::Int(a), BinOpKind::LessThanOrEqual, Value::Int(b)) => Value::Bool(a <= b),
                    (Value::Float(a), BinOpKind::LessThanOrEqual, Value::Float(b)) => Value::Bool(a <= b),
                    (Value::Int(a), BinOpKind::LessThanOrEqual, Value::Float(b)) => Value::Bool(a as f64 <= b),
                    (Value::Float(a), BinOpKind::LessThanOrEqual, Value::Int(b)) => Value::Bool(a <= b as f64),

                    (Value::Int(a), BinOpKind::Modulo, Value::Int(b)) => Value::Int(a % b),
                    (Value::Float(a), BinOpKind::Modulo, Value::Float(b)) => Value::Float(a % b),
                    (Value::Int(a), BinOpKind::Modulo, Value::Float(b)) => Value::Float(a as f64 % b),
                    (Value::Float(a), BinOpKind::Modulo, Value::Int(b)) => Value::Float(a % b as f64),


                    // TODO: add more bitwise operators
                    (Value::Int(a), BinOpKind::And, Value::Int(b)) => Value::Int(a & b),
                    (Value::Int(a), BinOpKind::Or, Value::Int(b)) => Value::Int(a | b),
                    (Value::Int(a), BinOpKind::BitwiseXor, Value::Int(b)) => Value::Int(a ^ b),

                    (Value::Int(a), BinOpKind::Power, Value::Int(b)) => Value::Int(a.pow(b as u32)),
                    (Value::Float(a), BinOpKind::Power, Value::Float(b)) => Value::Float(a.powf(b)),
                    (Value::Int(a), BinOpKind::Power, Value::Float(b)) => Value::Float((a as f64).powf(b)),
                    (Value::Float(a), BinOpKind::Power, Value::Int(b)) => Value::Float((a).powi(b as i32)),

                    _ => todo!("missing binary operator: {:?}", b.operator),
                };

                Ok(val)
            }

            _ => todo!("missing expr: {:?}", expr),
        };

        self.vm.push(val?);

        Ok(())
    }

    /// Finds a function by name.
    pub fn find_function(&self, name: &str) -> Option<&StoredFunction> {
        debug!("Looking for function: {}", name);

        self.functions.iter().find(|f| match f {
            StoredFunction::Native(n) => n.name == name,
            StoredFunction::Function { function, .. } => function.name == name,
        })
    }
}

impl Module {
    /// Executes a native function with the provided arguments.
    fn execute_native_function(&mut self, native: NativeFunction, args: Vec<Value>) -> Result<()> {
        debug!("Executing native function: {}", native.name);

        let mut params = vec![];
        for (param, val) in native.params.iter().zip(args.clone()) {
            if param.is_rest {
                let rest = args.iter().skip(native.params.len() - 1).cloned().collect();

                params.push(Value::Vec(rest));
            } else {
                params.push(val);
            }
        }

        let result = (native.func)(params);
        self.vm.push(result);

        Ok(())
    }

    /// Executes a user-defined function with the provided arguments.
    fn execute_user_defined_function(
        &mut self,
        function: Fn,
        defining_module: Arc<Mutex<Module>>,
        args: Vec<Value>,
        ctx: &Context,
    ) -> Result<()> {
        debug!("Executing user-defined function: {}", function.name);

        self.enter_scope();

        {
            let mut defining_module_guard = defining_module.lock().unwrap();
      

            for (param, arg) in function.params.iter().zip(args.iter().chain(std::iter::repeat(&Value::Null))) {
                let ident = param.ident.literal();
                if param.is_rest {
                    let rest = args.iter().skip(function.params.len() - 1).cloned().collect();
                    defining_module_guard.declare_variable(ident, Value::Vec(rest));
                } else {
                    defining_module_guard.declare_variable(ident, arg.clone());
                }
            }
        }

        let frame = Frame::new(
            function.name.clone(),
            function.fn_token.span.clone(),
            Frame::path_or_unknown(defining_module.lock().unwrap().path()),
        );
        self.vm.push_frame(frame);

        {
            let mut defining_module_guard = defining_module.lock().unwrap();
            for stmt in function.body.stmts {
                defining_module_guard.interpret_stmt(stmt, ctx)?;
            }
        }

        self.vm.pop_frame();
        self.exit_scope();

        let val = self.vm.pop().or(Some(Value::Void)).unwrap();
        self.vm.push(val);

        Ok(())
    }
}
