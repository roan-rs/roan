use crate::TextSpan;
use colored::Colorize;
use std::{fmt::Debug, path::PathBuf};

/// A frame represents a single function call.
///
/// It provides info of which function is being executed. This helps in debugging and error reporting.
#[derive(Clone)]
pub struct Frame {
    /// The name of the function being executed.
    pub name: String,
    /// The span of the function in the source code.
    pub span: TextSpan,
    /// The path of the file where the function is defined.
    pub path: String,
}

impl Frame {
    /// Creates a new `Frame` with the specified name, span, and path.
    ///
    /// # Parameters
    /// - `name` - The name of the function.
    /// - `span` - The span of the function in the source code.
    /// - `path` - The path of the file where the function is defined.
    ///
    /// # Returns
    /// The new `Frame`.
    ///
    /// # Examples
    /// ```rust
    /// use roan_error::{Position, TextSpan};
    /// use roan_error::frame::Frame;
    /// let frame = Frame::new("main", TextSpan::new(Position::new(1,1,1), Position::new(1,1,1), "main".into()), ".\\src\\main.roan");
    /// ```
    pub fn new(name: impl Into<String>, span: TextSpan, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            span,
            path: path.into(),
        }
    }

    /// If path is None returns "unknown" otherwise returns the path.
    pub fn path_or_unknown(path: Option<PathBuf>) -> String {
        let path = path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("unknown"));

        path.to_string_lossy().to_string()
    }
}

impl Debug for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "   {} {} {}{}{}{}{}",
            "at".dimmed(),
            self.name.bold(),
            self.path.cyan(),
            ":".dimmed(),
            self.span.start.line.to_string().yellow(),
            ":".dimmed(),
            self.span.start.column.to_string().dimmed().yellow()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Position;

    #[test]
    fn test_frame_new() {
        let frame = Frame::new(
            "main",
            TextSpan::new(Position::new(1, 1, 1), Position::new(1, 1, 1), "main".into()),
            ".\\src\\main.roan",
        );

        assert_eq!(frame.name, "main");
        assert_eq!(frame.span.start.line, 1);
        assert_eq!(frame.span.start.column, 1);
        assert_eq!(frame.path, ".\\src\\main.roan");
    }

    #[test]
    fn test_frame_path_or_unknown() {
        let path = Some(PathBuf::from("tests\\test.roan"));
        assert_eq!(Frame::path_or_unknown(path), "tests\\test.roan");

        let path = None;
        assert_eq!(Frame::path_or_unknown(path), "unknown");
    }
}