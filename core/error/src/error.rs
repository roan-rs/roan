use crate::{frame::Frame, span::TextSpan};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RoanError {
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
    #[error("Tried to import a item that does not exist: {0}")]
    ImportError(String, TextSpan),
    #[error("Failed to import {0}. {1}")]
    FailedToImportModule(String, String, TextSpan),
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
    #[error("{0}")]
    Throw(String, Vec<Frame>),
    #[error("Invalid escape sequence: {0}")]
    InvalidEscapeSequence(String, TextSpan),
    #[error("{0} does not evaluate to a boolean.")]
    NonBooleanCondition(String, TextSpan),
    #[error("Index out of bounds: {0} >= {1}")]
    IndexOutOfBounds(usize, usize, TextSpan),
    #[error("Type mismatch: {0}")]
    TypeMismatch(String, TextSpan),
    #[error("Invalid assignment {0}")]
    InvalidAssignment(String, TextSpan),
    #[error("Attempted to access non-existent property: {0}")]
    PropertyNotFoundError(String, TextSpan),
    #[error("Invalid property access")]
    InvalidPropertyAccess(TextSpan),
    #[error("Found break or continue statement outside of loop.")]
    InvalidBreakOrContinue(TextSpan),
    #[error("Break was used outside loop.")]
    LoopBreak(TextSpan),
    #[error("Continue was used outside loop.")]
    LoopContinue(TextSpan),
    #[error("Invalid spread operator usage.")]
    InvalidSpread(TextSpan),
    #[error("Found multiple 'self' parameters.")]
    MultipleSelfParameters(TextSpan),
    #[error("Found 'self' parameter in non-first position.")]
    SelfParameterNotFirst(TextSpan),
    #[error("Self parameter cannot be rest.")]
    SelfParameterCannotBeRest(TextSpan),
    #[error("Struct not found: {0}")]
    StructNotFoundError(String, TextSpan),
    #[error("Trait definition not found: {0}")]
    TraitNotFoundError(String, TextSpan),
    #[error("Struct {0} already implements trait {1}")]
    StructAlreadyImplementsTrait(String, String, TextSpan),
    #[error("Trait {0} doesn't implement required method")]
    TraitMethodNotImplemented(String, Vec<String>, TextSpan),
    #[error("Cannot assign value to static member")]
    StaticMemberAssignment(TextSpan),
    #[error("Attempted to access static member of non-struct type")]
    StaticMemberAccess(TextSpan),
    #[error("Only call expressions can be accessed in a static context")]
    StaticContext(TextSpan),
    #[error("Invalid unary operator: {0}")]
    InvalidUnaryOperation(String, TextSpan),
    #[error("Missing non-nullable parameter: {0}")]
    MissingParameter(String, TextSpan),
    #[error("Invalid type provided: {0}. Available types: {1}")]
    InvalidType(String, String, TextSpan),
    #[error("Missing field: {0} required by struct: {1}")]
    MissingField(String, String, TextSpan),
    #[error("Expected {0} arguments to {1} function, got {2}")]
    TooManyArguments(usize, String, usize, TextSpan),
}

pub fn get_span_from_err(err: &RoanError) -> Option<TextSpan> {
    match err {
        RoanError::Io(_) | RoanError::ResolverError(_) | RoanError::ModuleError(_) => None,
        RoanError::RestParameterNotLast(span)
        | RoanError::RestParameterNotLastPosition(span)
        | RoanError::MultipleRestParameters(span)
        | RoanError::SelfParameterCannotBeRest(span)
        | RoanError::SelfParameterNotFirst(span)
        | RoanError::MultipleSelfParameters(span)
        | RoanError::StaticContext(span)
        | RoanError::StaticMemberAccess(span)
        | RoanError::StaticMemberAssignment(span) => Some(span.clone()),
        RoanError::InvalidToken(_, span)
        | RoanError::SemanticError(_, span)
        | RoanError::UnexpectedToken(_, span)
        | RoanError::InvalidEscapeSequence(_, span)
        | RoanError::NonBooleanCondition(_, span)
        | RoanError::StructNotFoundError(_, span)
        | RoanError::TraitNotFoundError(_, span) => Some(span.clone()),
        RoanError::TraitMethodNotImplemented(_, _, span)
        | RoanError::StructAlreadyImplementsTrait(_, _, span)
        | RoanError::ExpectedToken(_, _, span)
        | RoanError::FailedToImportModule(_, _, span)
        | RoanError::MissingField(_, _, span)
        | RoanError::InvalidType(_, _, span)
        | RoanError::IndexOutOfBounds(_, _, span) => Some(span.clone()),
        RoanError::UndefinedFunctionError(_, span)
        | RoanError::VariableNotFoundError(_, span)
        | RoanError::ImportError(_, span)
        | RoanError::PropertyNotFoundError(_, span)
        | RoanError::TypeMismatch(_, span)
        | RoanError::InvalidAssignment(_, span)
        | RoanError::MissingParameter(_, span)
        | RoanError::InvalidUnaryOperation(_, span) => Some(span.clone()),
        RoanError::InvalidPropertyAccess(span)
        | RoanError::InvalidSpread(span)
        | RoanError::InvalidBreakOrContinue(span)
        | RoanError::LoopBreak(span)
        | RoanError::LoopContinue(span) => Some(span.clone()),
        RoanError::TooManyArguments(_, _, _, span) => Some(span.clone()),
        _ => None,
    }
}
