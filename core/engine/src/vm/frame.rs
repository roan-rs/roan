use std::path::{Path, PathBuf};
use roan_error::TextSpan;

/// A frame represents a single function call.
///
/// It provides info of which function is being executed. This helps in debugging and error reporting.
#[derive(Debug, Clone)]
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
    /// use roan_engine::vm::Frame;
    /// use roan_error::{Position, TextSpan};
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
    pub fn path_or_unknown(
        path: Option<PathBuf>
    ) -> String {
        let path = path.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("unknown"));

        path.to_string_lossy().to_string()
    }
}