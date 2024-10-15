use crate::{error::PulseError, span::TextSpan};
use colored::Colorize;
use log::Level;
use std::io::{BufWriter, Stderr, Write};

/// Represents a diagnostic message, which includes information about an error or warning
/// and can be pretty-printed to the console.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// The title or summary of the diagnostic message.
    pub title: String,
    /// An optional detailed description of the diagnostic message.
    pub text: Option<String>,
    /// The severity level of the diagnostic message (e.g., Error, Warning).
    pub level: Level,
    /// The location in the source code where the error or warning occurred, represented as a `TextSpan`.
    pub location: Option<TextSpan>,
    /// An optional hint that provides additional guidance on resolving the issue.
    pub hint: Option<String>,
    /// The content of the source code related to the diagnostic.
    pub content: Option<String>,
}

impl Diagnostic {
    /// Logs the diagnostic in a human-readable format to the provided buffer.
    ///
    /// The message is colored according to its severity level, and the source code around
    /// the error location (if available) is highlighted.
    ///
    /// # Arguments
    ///
    /// * `buff` - A mutable reference to a `BufWriter` that writes to `stderr`.
    ///
    /// # Example
    ///
    /// ```rust ignore
    /// use std::io::BufWriter;
    /// use log::Level;
    /// use roan_error::{Diagnostic, Position, TextSpan};
    /// let diagnostic = Diagnostic {
    ///     title: "Syntax Error".to_string(),
    ///     text: None,
    ///     level: Level::Error,
    ///     location: Some(TextSpan::new(Position::new(1, 1, 0), Position::new(1, 5, 4), "test".to_string())),
    ///     hint: None,
    ///     content: Some("let x = ;".to_string()),
    /// };
    ///
    /// let mut buff = BufWriter::new(std::io::stderr());
    /// diagnostic.log_pretty(&mut buff);
    /// ```
    pub fn log_pretty(&self, buff: &mut BufWriter<Stderr>) {
        writeln!(
            buff,
            "{}{}{}",
            self.level.to_string().to_lowercase().bright_red(),
            ": ".dimmed(),
            self.title
        )
        .expect("Error writing level");

        if let Some(location) = &self.location {
            if let Some(content) = &self.content {
                let line = content
                    .lines()
                    .nth(location.start.line as usize)
                    .expect("Error getting line");
                let line_number = location.start.line;
                let column = location.start.column;
                let line_content = line.trim_end();
                let decoration =
                    "^".repeat(location.end.column as usize - location.start.column as usize);

                writeln!(buff, "{} {}:{}", "--->".cyan(), line_number, column)
                    .expect("Error writing line number");

                if line_number > 1 {
                    let line_before = format!("{} |", line_number - 1);
                    writeln!(buff, "{}", line_before.cyan()).expect("Error writing line number");
                }

                let line_current = format!("{} |", line_number);
                write!(buff, "{}", line_current.cyan()).expect("Error writing line number");
                writeln!(buff, "    {}", line_content).expect("Error writing content");

                let padding_left =
                    " ".repeat((column + 6 + line_number.to_string().len() as u32) as usize);
                writeln!(buff, "{}{}", padding_left, decoration.bright_red())
                    .expect("Error writing decoration");

                if line_number > 1 {
                    let line_after = format!("{} |", line_number + 1);
                    writeln!(buff, "{}", line_after.cyan()).expect("Error writing line number");
                }
            }
        }

        if let Some(text) = &self.text {
            writeln!(buff, "{}", text).expect("Error writing text");
        }

        self.print_hint(buff);
    }

    /// Prints a hint message (if available) to the provided buffer.
    ///
    /// # Arguments
    ///
    /// * `buff` - A mutable reference to a `BufWriter` that writes to `stderr`.
    pub fn print_hint(&self, buff: &mut BufWriter<Stderr>) {
        if let Some(hint) = &self.hint {
            writeln!(buff, "{}{}", "Hint: ".bright_cyan(), hint.bright_cyan())
                .expect("Error writing hint");
        }
    }
}

/// Prints a diagnostic message based on the provided error. The function matches
/// the error type with corresponding diagnostics and logs it prettily.
///
/// # Arguments
///
/// * `err` - An `anyhow::Error` object that encapsulates the actual error.
/// * `content` - An optional string slice containing the source code related to the error.
///
/// # Example
///
/// ```rust ignore
/// use roan_error::error::PulseError;
/// use roan_error::print_diagnostic;
/// let err = PulseError::SemanticError("Unexpected token".to_string(), span);
/// print_diagnostic(anyhow::Error::new(err), Some(source_code));
/// ```
pub fn print_diagnostic(err: anyhow::Error, content: Option<String>) {
    let pulse_error = err.downcast_ref::<PulseError>();

    if let Some(err) = pulse_error {
        let err_str = err.to_string();

        if let PulseError::Throw(content, frames) = err {
            let mut buff = BufWriter::new(std::io::stderr());
            write!(buff, "{}{}", "error".bright_red(), ": ".dimmed()).expect("Error writing level");
            writeln!(buff, "{}", content).expect("Error writing text");
            for frame in frames {
                writeln!(buff, "{:?}", frame).expect("Error writing text");
            }
            return;
        }

        let diagnostic = match err {
            PulseError::Io(_) => Diagnostic {
                title: "IO error".to_string(),
                text: Some(err_str),
                level: Level::Error,
                location: None,
                hint: None,
                content: None,
            },
            PulseError::RestParameterNotLast(span)
            | PulseError::RestParameterNotLastPosition(span)
            | PulseError::MultipleRestParameters(span)
            | PulseError::SelfParameterCannotBeRest(span)
            | PulseError::SelfParameterNotFirst(span)
            | PulseError::MultipleSelfParameters(span)
            | PulseError::StaticContext(span)
            | PulseError::StaticMemberAccess(span)
            | PulseError::StaticMemberAssignment(span) => Diagnostic {
                title: err_str,
                text: None,
                level: Level::Error,
                location: Some(span.clone()),
                hint: None,
                content,
            },
            PulseError::InvalidToken(_, span)
            | PulseError::SemanticError(_, span)
            | PulseError::UnexpectedToken(_, span)
            | PulseError::InvalidEscapeSequence(_, span)
            | PulseError::NonBooleanCondition(_, span)
            | PulseError::StructNotFoundError(_, span)
            | PulseError::TraitNotFoundError(_, span) => Diagnostic {
                title: err_str,
                text: None,
                level: Level::Error,
                location: Some(span.clone()),
                hint: None,
                content,
            },
            PulseError::TraitMethodNotImplemented(name, methods, span) => Diagnostic {
                title: format!(
                    "Trait {name} doesn't implement these methods: {}",
                    methods.join(", ")
                ),
                text: None,
                level: Level::Error,
                location: Some(span.clone()),
                hint: Some("Method not implemented".to_string()),
                content,
            },
            PulseError::StructAlreadyImplementsTrait(_, _, span) => Diagnostic {
                title: err_str,
                text: None,
                level: Level::Error,
                location: Some(span.clone()),
                hint: Some("Struct already implements this trait".to_string()),
                content,
            },
            PulseError::ExpectedToken(expected, hint, span) => Diagnostic {
                title: format!("Expected {}", expected),
                text: None,
                level: Level::Error,
                location: Some(span.clone()),
                hint: Some(hint.clone()),
                content,
            },
            PulseError::ResolverError(_) => Diagnostic {
                title: err_str,
                text: None,
                level: Level::Error,
                location: None,
                hint: None,
                content: None,
            },
            PulseError::ModuleError(_) => Diagnostic {
                title: err_str,
                text: None,
                level: Level::Error,
                location: None,
                hint: None,
                content: None,
            },
            PulseError::ModuleNotFoundError(_, span)
            | PulseError::UndefinedFunctionError(_, span)
            | PulseError::VariableNotFoundError(_, span)
            | PulseError::ImportError(_, span)
            | PulseError::PropertyNotFoundError(_, span)
            | PulseError::TypeMismatch(_, span)
            | PulseError::InvalidAssignment(_, span) => Diagnostic {
                title: err_str,
                text: None,
                level: Level::Error,
                location: Some(span.clone()),
                hint: None,
                content,
            },
            PulseError::InvalidBreakOrContinue(span) => Diagnostic {
                title: err_str,
                text: None,
                level: Level::Error,
                location: Some(span.clone()),
                hint: Some(
                    "Break and continue statements can only be used inside loops".to_string(),
                ),
                content,
            },
            PulseError::LoopBreak(span) | PulseError::LoopContinue(span) => Diagnostic {
                title: err_str,
                text: None,
                level: Level::Error,
                location: Some(span.clone()),
                hint: Some(
                    "Break and continue statements can only be used inside loops".to_string(),
                ),
                content,
            },
            PulseError::InvalidSpread(span) => Diagnostic {
                title: err_str,
                text: None,
                level: Level::Error,
                location: Some(span.clone()),
                hint: Some(
                    "Spread operator can only be used in function calls or vectors".to_string(),
                ),
                content,
            },
            PulseError::InvalidPropertyAccess(span) => Diagnostic {
                title: err_str,
                text: None,
                level: Level::Error,
                location: Some(span.clone()),
                hint: Some("Only string literals or call expressions are allowed".to_string()),
                content,
            },
            PulseError::IndexOutOfBounds(_, _, span) => Diagnostic {
                title: err_str,
                text: None,
                level: Level::Error,
                location: Some(span.clone()),
                hint: None,
                content,
            },
            _ => {
                log::error!("{:?}", err);
                return;
            }
        };

        let mut buff = BufWriter::new(std::io::stderr());
        diagnostic.log_pretty(&mut buff);
    } else {
        log::error!("{:?}", err);
    }
}
