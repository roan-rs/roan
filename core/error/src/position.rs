use std::fmt;

/// Represents a position in a text, consisting of line, column, and byte index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    /// The current line number (starting from 1).
    pub line: u32,
    /// The current column number (starting from 1).
    pub column: u32,
    /// The current byte index in the text (starting from 0).
    pub index: usize,
}

impl Position {
    /// Creates a new `Position` with the given line, column, and index.
    ///
    /// # Arguments
    ///
    /// * `line` - The line number (1-based).
    /// * `column` - The column number (1-based).
    /// * `index` - The byte index in the text (0-based).
    ///
    /// # Example
    ///
    /// ```
    /// let pos = Position::new(1, 1, 0);
    /// assert_eq!(pos.line(), 1);
    /// assert_eq!(pos.column(), 1);
    /// assert_eq!(pos.index(), 0);
    /// ```
    pub fn new(line: u32, column: u32, index: usize) -> Self {
        Self {
            line,
            column,
            index,
        }
    }

    /// Increments the line number, resets the column to 1, and increments the index by 1.
    pub fn increment_line(&mut self) {
        self.line += 1;
        self.column = 1;
        self.index += 1;
    }

    /// Increments the column number and the index by 1.
    pub fn increment_column(&mut self) {
        self.column += 1;
        self.index += 1;
    }

    /// Returns the current line number.
    ///
    /// # Returns
    ///
    /// The line number (1-based).
    pub fn line(&self) -> u32 {
        self.line
    }

    /// Returns the current column number.
    ///
    /// # Returns
    ///
    /// The column number (1-based).
    pub fn column(&self) -> u32 {
        self.column
    }

    /// Returns the current byte index.
    ///
    /// # Returns
    ///
    /// The byte index (0-based).
    pub fn index(&self) -> usize {
        self.index
    }
}

impl fmt::Display for Position {
    /// Formats the position as `line:column (index: byte_index)`.
    ///
    /// # Example
    ///
    /// ```
    /// let pos = Position::new(1, 1, 0);
    /// assert_eq!(format!("{}", pos), "1:1 (index: 0)");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{} (index: {})", self.line, self.column, self.index)
    }
}

impl Default for Position {
    /// Returns a default `Position`, starting at line 1, column 1, and index 0.
    ///
    /// # Example
    ///
    /// ```
    /// let default_pos = Position::default();
    /// assert_eq!(default_pos, Position::new(1, 1, 0));
    /// ```
    fn default() -> Self {
        Self::new(1, 1, 0)
    }
}
