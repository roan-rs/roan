use crate::span::TextSpan;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PulseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid token: {0}")]
    InvalidToken(String, TextSpan),
    #[error("Expected {0}.")]
    ExpectedToken(String, String, TextSpan),
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String, TextSpan),
    #[error("{0}")]
    SemanticError(String, TextSpan),
    #[error("Semantic error: {0}")]
    ResolverError(String),
    #[error("{0}")]
    ModuleError(String),
    #[error("Tried to import a function that does not exist: {0}")]
    ImportError(String, TextSpan),
    #[error("Tried to import module that does not exist: {0}")]
    ModuleNotFoundError(String, TextSpan),
    #[error("Couldn't find variable: {0}")]
    VariableNotFoundError(String, TextSpan),
    #[error("Call to undefined function: {0}")]
    UndefinedFunctionError(String, TextSpan),
    #[error("Found normal parameter after rest parameter.")]
    RestParameterNotLast(TextSpan),
    #[error("Found rest parameter in non-last position.")]
    RestParameterNotLastPosition(TextSpan),
    #[error("Found more than one rest parameter.")]
    MultipleRestParameters(TextSpan),
}
