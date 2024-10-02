use std::fmt::Debug;
use std::path::{Path, PathBuf};
use roan_ast::source::{Source};
use anyhow::Result;
use log::debug;
use roan_ast::{Lexer, Parser, Token, Fn, Let, Stmt, Use, Ast, Expr, BinOpKind, Variable};
use roan_error::error::PulseError::{ImportError, ModuleNotFoundError, VariableNotFoundError};
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
    variables: Vec<(Value, Variable)>,
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

        Self { source, path, tokens: vec![], functions: vec![], exports: vec![], imports: vec![], variables: vec![], ast: Ast::new(), vm: VM::new() }
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

    /// Interpret statement from the module.
    pub fn interpret_stmt(&mut self, stmt: Stmt, ctx: &Context) -> Result<()> {
        match stmt {
            Stmt::Fn(f) => {
                debug!("Interpreting function: {}", f.name);
                self.functions.push(f.clone());

                if f.exported {
                    self.exports.push((f.name.clone(), ExportType::Function(f.clone())));
                }
            }
            Stmt::Use(u) => {
                debug!("Interpreting use: {}", u.from.literal());
                let mut module = ctx.module_loader.load(&self, &u.from.literal(), ctx)
                    .map_err(|e| ModuleNotFoundError(u.from.literal(), u.from.span.clone()))?;
                module.parse()?;
                module.interpret(ctx)?;

                let imported_items: Vec<(String, &Token)> = u.items.iter().map(|i| (i.literal(), i)).collect::<Vec<_>>();

                for (name, item) in imported_items {
                    match module.find_function(&name) {
                        Some(f) => {
                            self.functions.push(f.clone());
                        }
                        None => Err(ImportError(name, item.span.clone()))?,
                    }
                }
            }
            Stmt::Let(l) => {
                debug!("Interpreting let: {:?}", l.ident);
                self.interpret_expr(l.initializer.as_ref(), ctx)?;

                let var = Variable { token: l.ident.clone(), ident: l.ident.literal() };
                self.variables.push((self.vm.pop().unwrap(), var));
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
            _ => {}
        }

        Ok(())
    }

    pub fn interpret_expr(&mut self, expr: &Expr, ctx: &Context) -> Result<()> {
        let val: Result<Value> = match expr {
            Expr::Variable(v) => {
                debug!("Interpreting variable: {}", v.ident);

                let variable = self.find_variable(&v.ident)
                    .ok_or_else(|| VariableNotFoundError(
                        v.ident.clone(),
                        v.token.span.clone(),
                    ))?;

                Ok(variable.0.clone())
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
                    let func = self.find_function(&call.callee)
                        .ok_or_else(|| ImportError(call.callee.clone(), call.token.span.clone()))?;
                    func.clone()
                };

                for (expr, val) in function.params.iter().zip(
                    args
                ) {
                    let var = Variable { token: expr.ident.clone(), ident: expr.ident.literal() };

                    self.variables.push((val, var));
                }

                let frame = Frame::new(
                    call.callee.clone(),
                    call.token.span.clone(),
                    Frame::path_or_unknown(self.path()),
                );
                self.vm.push_frame(frame);
                self.vm.push_frame(
                    Frame::new(function.name.clone(), function.fn_token.span.clone(), Frame::path_or_unknown(self.path()))
                );

                for stmt in function.body.stmts {
                    self.interpret_stmt(stmt, ctx)?;
                }

                self.vm.pop_frame();
                self.vm.pop_frame();

                let val = self.vm.pop()
                    .ok_or_else(|| VariableNotFoundError(call.callee.clone(), call.token.span.clone()))?;
                println!("val: {:?}", val);
                Ok(val)
            }
            Expr::Binary(b) => {
                debug!("Interpreting binary: {:?}", b);

                self.interpret_expr(&b.left, ctx)?;
                self.interpret_expr(&b.right, ctx)?;
                let left = self.vm.pop().unwrap();
                let right = self.vm.pop().unwrap();

                let val = match b.operator {
                    BinOpKind::Plus => left + right,
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

    /// Looks for a variable with the specified name.
    pub fn find_variable(&self, name: &str) -> Option<(Value, &Variable)> {
        debug!("Looking for variable: {}", name);
        self.variables.iter().find(|v| v.1.ident == name).map(|(val, let_var)| (val.clone(), let_var))
    }
}
