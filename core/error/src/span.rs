use crate::position::Position;

/// Represents a span of text between two positions, including the literal text.
#[derive(Clone, PartialEq, Eq)]
pub struct TextSpan {
    /// The starting position of the text span.
    pub start: Position,
    /// The ending position of the text span.
    pub end: Position,
    /// The literal text contained in the span.
    pub literal: String,
}

impl TextSpan {
    /// Creates a new `TextSpan` from a starting position, an ending position, and a literal string.
    ///
    /// # Arguments
    ///
    /// * `start` - The starting position of the text span.
    /// * `end` - The ending position of the text span.
    /// * `literal` - The literal text represented by the span.
    ///
    /// # Example
    ///
    /// ```
    /// let start = Position::new(1, 1, 0);
    /// let end = Position::new(1, 5, 4);
    /// let span = TextSpan::new(start, end, "test".to_string());
    /// assert_eq!(span.length(), 4);
    /// ```
    pub fn new(start: Position, end: Position, literal: String) -> Self {
        Self { start, end, literal }
    }

    /// Combines multiple `TextSpan` objects into one. The spans are sorted by their starting positions.
    ///
    /// # Panics
    ///
    /// Panics if the input vector is empty.
    ///
    /// # Arguments
    ///
    /// * `spans` - A vector of `TextSpan` objects to combine.
    ///
    /// # Returns
    ///
    /// A new `TextSpan` that spans from the start of the first span to the end of the last span,
    /// with the concatenated literal text.
    ///
    /// # Example
    ///
    /// ```
    /// let span1 = TextSpan::new(Position::new(1, 1, 0), Position::new(1, 5, 4), "test".to_string());
    /// let span2 = TextSpan::new(Position::new(1, 6, 5), Position::new(1, 10, 9), "span".to_string());
    /// let combined = TextSpan::combine(vec![span1, span2]);
    /// assert_eq!(combined.literal, "testspan");
    /// ```
    pub fn combine(mut spans: Vec<TextSpan>) -> TextSpan {
        if spans.is_empty() {
            panic!("Cannot combine empty spans")
        }
        spans.sort_by(|a, b| a.start.index.cmp(&b.start.index));

        let start = spans.first().unwrap().start;
        let end = spans.last().unwrap().end;

        TextSpan::new(
            start,
            end,
            spans.into_iter().map(|span| span.literal).collect(),
        )
    }

    /// Returns the length of the span, calculated as the difference between the end and start indices.
    ///
    /// # Returns
    ///
    /// The length of the text span in bytes.
    pub fn length(&self) -> usize {
        self.end.index - self.start.index
    }

    /// Extracts the literal text from the given input string based on the start and end positions.
    ///
    /// # Arguments
    ///
    /// * `input` - The input string from which to extract the literal text.
    ///
    /// # Returns
    ///
    /// A slice of the input string that corresponds to the span's range.
    pub fn literal<'a>(&self, input: &'a str) -> &'a str {
        &input[self.start.index..self.end.index]
    }
}

impl std::fmt::Debug for TextSpan {
    /// Formats the `TextSpan` as `"literal" (line:column)`.
    ///
    /// # Example
    ///
    /// ```
    /// let span = TextSpan::new(Position::new(1, 1, 0), Position::new(1, 5, 4), "test".to_string());
    /// assert_eq!(format!("{:?}", span), "\"test\" (1:1)");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "\"{}\" ({}:{})",
            self.literal, self.start.line, self.start.column
        )
    }
}
