use crate::{
    context::Context,
    interpreter::passes::{imports::ImportPass, resolver::ResolverPass, types::TypePass, Pass},
    natives::get_stored_function,
    value::Value,
    vm::{native_fn::NativeFunction, VM},
};
use anyhow::Result;
use indexmap::IndexMap;
use roan_ast::{
    source::Source, Ast, Expr, Fn, Lexer, Parser, StructField, StructImpl, Token, TraitDef,
    TraitImpl,
};
use roan_error::{error::RoanError::VariableNotFoundError, print_diagnostic, TextSpan};
use std::{
    collections::HashMap,
    fmt::Debug,
    path::{Path, PathBuf},
};
use tracing::debug;
use uuid::Uuid;

pub mod loaders;

#[derive(Clone, Debug)]
pub struct StoredStruct {
    pub defining_module: String,
    pub struct_token: Token,
    pub name: Token,
    pub fields: IndexMap<String, StructField>,
    pub public: bool,
    pub impls: Vec<StoredImpl>,
    pub trait_impls: Vec<StoredTraitImpl>,
}

impl StoredStruct {
    fn find_method_internal(&self, name: &str, is_static: bool) -> Option<&Fn> {
        self.impls
            .iter()
            .flat_map(|impl_stmt| impl_stmt.def.methods.iter())
            .chain(
                self.trait_impls
                    .iter()
                    .flat_map(|impl_stmt| impl_stmt.def.methods.iter()),
            )
            .find(|method| method.name == name && method.is_static == is_static)
    }

    pub fn find_static_method(&self, name: &str) -> Option<&Fn> {
        self.find_method_internal(name, true)
    }

    pub fn find_method(&self, name: &str) -> Option<&Fn> {
        self.find_method_internal(name, false)
    }

    pub fn find_field(&self, name: &str) -> Option<&StructField> {
        self.fields.get(name)
    }
}

#[derive(Clone, Debug)]
pub struct StoredImpl {
    pub def: StructImpl,
    pub defining_module: String,
}

#[derive(Clone, Debug)]
pub struct StoredTraitImpl {
    pub def: TraitImpl,
    pub defining_module: String,
}

#[derive(Clone, Debug)]
pub struct StoredConst {
    pub ident: Token,
    pub value: Value,
    pub defining_module: String,
}

#[derive(Debug, Clone)]
pub enum ExportType {
    Function(Fn),
    Trait(TraitDef),
    Struct(StoredStruct),
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
    pub source: Source,
    pub path: Option<PathBuf>,
    pub tokens: Vec<Token>,
    pub ast: Ast,
    pub functions: Vec<StoredFunction>,
    pub exports: Vec<(String, ExportType)>,
    pub scopes: Vec<HashMap<String, Value>>,
    pub structs: Vec<StoredStruct>,
    pub traits: Vec<TraitDef>,
    pub consts: Vec<StoredConst>,
    pub id: String,
    pub lex_comments: bool,
    pub passes: Vec<Box<dyn Pass>>,
}

impl Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module")
            .field("path", &self.path)
            // .field("source", &self.source)
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
    /// An `Module` containing the new Module.
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
            lex_comments: false,
            passes: vec![
                Box::new(ImportPass {}),
                Box::new(ResolverPass {}),
                Box::new(TypePass::new()),
            ],
        }
    }

    pub fn set_lex_comments(&mut self, lex_comments: bool) {
        self.lex_comments = lex_comments;
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
    pub fn parse(&mut self, ctx: &mut Context, vm: &mut VM) -> Result<()> {
        debug!("Parsing module from source");
        let mut lexer = Lexer::new(self.source.clone());

        let tokens = lexer.lex(self.lex_comments)?;
        debug!("Parsed {} tokens", tokens.len());
        self.tokens = tokens;

        let mut parser = Parser::new(self.tokens.clone());

        debug!("Parsing tokens into AST");
        let ast = parser.parse()?;
        self.ast = ast;

        let mut passes = self.passes.clone();
        for pass in passes.iter_mut() {
            pass.run(self, ctx, vm)?;
        }

        Ok(())
    }

    pub fn interpret(&mut self, ctx: &mut Context, vm: &mut VM) -> Result<()> {
        for stmt in self.ast.stmts.clone() {
            match self.interpret_stmt(stmt, ctx, vm) {
                Ok(_) => {}
                Err(e) => {
                    print_diagnostic(&e, Some(self.source.content()), self.path());
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
