use crate::position::Position;

#[derive(Clone, PartialEq, Eq)]
pub struct TextSpan {
    pub start: Position,
    pub end: Position,
    pub literal: String,
}

impl TextSpan {
    pub fn new(start: Position, end: Position, literal: String) -> Self {
        Self {
            start,
            end,
            literal,
        }
    }

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

    pub fn length(&self) -> usize {
        self.end.index - self.start.index
    }

    pub fn literal<'a>(&self, input: &'a str) -> &'a str {
        &input[self.start.index..self.end.index]
    }
}

impl std::fmt::Debug for TextSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "\"{}\" ({}:{})",
            self.literal, self.start.line, self.start.column
        )
    }
}
