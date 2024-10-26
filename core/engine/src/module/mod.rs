use crate::{
    context::Context,
    natives::get_stored_function,
    value::Value,
    vm::{native_fn::NativeFunction, VM},
};
use anyhow::Result;
use roan_ast::{
    source::Source, Ast, Const, Expr, Fn, Lexer, Parser, Stmt, Struct, Token, TraitDef, Variable,
};
use roan_error::{error::PulseError::VariableNotFoundError, print_diagnostic, TextSpan};
use std::{
    collections::HashMap,
    fmt::Debug,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tracing::debug;
use uuid::Uuid;

pub mod loaders;

#[derive(Clone, Debug)]
pub struct StoredStruct {
    pub(crate) def: Struct,
    pub(crate) defining_module: String,
}

#[derive(Clone, Debug)]
pub struct StoredConst {
    pub ident: Token,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub enum ExportType {
    Function(Fn),
    Trait(TraitDef),
    Struct(Struct),
    Const(StoredConst),
}

/// Represents a function stored in a module.
#[derive(Debug, Clone)]
pub enum StoredFunction {
    Native(NativeFunction),
    Function {
        function: Fn,
        defining_module: String,
    },
}

#[derive(Clone)]
pub struct Module {
    pub(crate) source: Source,
    pub(crate) path: Option<PathBuf>,
    pub(crate) tokens: Vec<Token>,
    pub(crate) ast: Ast,
    pub(crate) functions: Vec<StoredFunction>,
    pub(crate) exports: Vec<(String, ExportType)>,
    pub(crate) scopes: Vec<HashMap<String, Value>>,
    pub(crate) structs: Vec<StoredStruct>,
    pub(crate) traits: Vec<TraitDef>,
    pub(crate) consts: Vec<StoredConst>,
    pub(crate) id: String,
}

impl Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module")
            .field("path", &self.path)
            .field("source", &self.source)
            // .field("tokens", &self.tokens)
            // .field("ast", &self.ast)
            // .field("functions", &self.functions)
            .field("exports", &self.exports)
            .field("scopes", &self.scopes)
            .field("structs", &self.structs)
            .field("traits", &self.traits)
            .field("consts", &self.consts)
            .finish()
    }
}

impl Module {
    /// Creates a new Module from the specified Source.
    ///
    /// # Parameters
    /// - source - The source of the module.
    ///
    /// # Returns
    /// An `Arc<Mutex<Self>>` containing the new Module.
    pub fn new(source: Source) -> Self {
        let path = source.path().as_deref().map(Path::to_path_buf);

        Self {
            source,
            path,
            tokens: vec![],
            functions: get_stored_function(),
            exports: vec![],
            scopes: vec![HashMap::new()],
            ast: Ast::new(),
            structs: vec![],
            traits: vec![],
            consts: vec![],
            id: Uuid::new_v4().to_string(),
        }
    }

    /// Get module id
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Returns the path of the module.
    pub fn path(&self) -> Option<PathBuf> {
        self.path.clone()
    }

    /// Returns the source of the module.
    pub fn source(&self) -> &Source {
        &self.source
    }

    /// Returns tokens of the module.
    pub fn tokens(&self) -> &Vec<Token> {
        &self.tokens
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
        self.tokens = vec![];

        Ok(())
    }

    pub fn interpret(&mut self, ctx: &mut Context, vm: &mut VM) -> Result<()> {
        for stmt in self.ast.stmts.clone() {
            match self.interpret_stmt(stmt, ctx, vm) {
                Ok(_) => {}
                Err(e) => {
                    print_diagnostic(e, Some(self.source.content()));
                    std::process::exit(1);
                }
            }
        }

        Ok(())
    }

    /// Enter a new scope by pushing a new HashMap onto the scopes stack.
    pub fn enter_scope(&mut self) {
        debug!("Entering new scope");
        self.scopes.push(HashMap::new());
    }

    /// Exit the current scope by popping the top HashMap from the scopes stack.
    pub fn exit_scope(&mut self) {
        debug!("Exiting current scope");
        self.scopes.pop();
    }

    /// Declare a new variable in the current (innermost) scope.
    pub fn declare_variable(&mut self, name: String, val: Value) {
        debug!("Declaring variable '{}' in current scope", name);
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, val);
        }
    }

    /// Set an existing variable's value in the nearest enclosing scope.
    pub fn set_variable(&mut self, name: &str, val: Value) -> Result<()> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                debug!("Setting variable '{}' to {:?}", name, val);
                scope.insert(name.to_string(), val);
                return Ok(());
            }
        }
        // Variable not found in any scope
        Err(VariableNotFoundError(name.to_string(), TextSpan::default()).into())
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

    /// Finds a constant by name.
    pub fn find_const(&self, name: &str) -> Option<&StoredConst> {
        self.consts.iter().find(|c| c.ident.literal() == name)
    }

    pub fn name(&self) -> String {
        self.path()
            .unwrap()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }

    pub fn extract_variable_name(expr: &Expr) -> Option<String> {
        match expr {
            Expr::Variable(v) => Some(v.ident.clone()),
            Expr::Access(access) => Self::extract_variable_name(&access.base),
            _ => None,
        }
    }

    /// Finds a function by name.
    pub fn find_function(&self, name: &str) -> Option<&StoredFunction> {
        debug!("Looking for function: {}", name);

        self.functions.iter().find(|f| match f {
            StoredFunction::Native(n) => n.name == name,
            StoredFunction::Function { function, .. } => function.name == name,
        })
    }

    pub fn update_variable(
        &mut self,
        name: &str,
        val: Value,
        func: fn(Value, Value) -> Value,
    ) -> Result<()> {
        let variable = self
            .find_variable(name)
            .ok_or_else(|| VariableNotFoundError(name.to_string(), TextSpan::default()))?;

        let new_val = func(variable.clone(), val);
        self.set_variable(name, new_val)?;
        Ok(())
    }
}
