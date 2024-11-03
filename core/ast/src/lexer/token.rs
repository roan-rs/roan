use roan_error::span::TextSpan;
use std::{
    fmt,
    fmt::{Debug, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: TextSpan,
}

impl Token {
    /// Creates a new `Token` with the specified `TokenKind` and `TextSpan`.
    pub fn new(kind: TokenKind, span: TextSpan) -> Self {
        Self { kind, span }
    }

    /// Returns the literal value of the token.
    pub fn literal(&self) -> String {
        self.span.literal.clone()
    }

    /// Checks if the token is a string.
    pub fn is_string(&self) -> bool {
        matches!(self.kind, TokenKind::String(_))
    }

    /// Tries to convert the token to a boolean.
    ///
    /// Throws a panic if the token is not a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self.kind {
            TokenKind::True => Some(true),
            TokenKind::False => Some(false),
            _ => unreachable!("Token is not a boolean"),
        }
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            // Separators
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::DoubleDot => write!(f, ".."),
            TokenKind::TripleDot => write!(f, "..."),
            TokenKind::DoubleColon => write!(f, "::"),

            // Literals
            TokenKind::Identifier => write!(f, "Identifier"),
            TokenKind::String(s) => write!(f, "{}", s),
            TokenKind::Float(r) => write!(f, "{}", r),
            TokenKind::Integer(i) => write!(f, "{}", i),
            TokenKind::Char(c) => write!(f, "{}", c),

            // Keywords
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::While => write!(f, "while"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Use => write!(f, "use"),
            TokenKind::Pub => write!(f, "pub"),
            TokenKind::From => write!(f, "from"),
            TokenKind::Throw => write!(f, "throw"),
            TokenKind::Try => write!(f, "try"),
            TokenKind::Catch => write!(f, "catch"),
            TokenKind::Loop => write!(f, "loop"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Null => write!(f, "null"),
            TokenKind::Impl => write!(f, "impl"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Trait => write!(f, "trait"),
            TokenKind::Then => write!(f, "then"),
            TokenKind::Const => write!(f, "const"),

            // Operators
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Asterisk => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Equals => write!(f, "="),
            TokenKind::Ampersand => write!(f, "&"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Caret => write!(f, "^"),
            TokenKind::DoubleAsterisk => write!(f, "**"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Tilde => write!(f, "~"),
            TokenKind::GreaterThan => write!(f, ">"),
            TokenKind::LessThan => write!(f, "<"),
            TokenKind::GreaterThanEquals => write!(f, ">="),
            TokenKind::LessThanEquals => write!(f, "<="),
            TokenKind::EqualsEquals => write!(f, "=="),
            TokenKind::BangEquals => write!(f, "!="),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::And => write!(f, "&&"),
            TokenKind::Or => write!(f, "||"),
            TokenKind::Increment => write!(f, "++"),
            TokenKind::Decrement => write!(f, "--"),
            TokenKind::MinusEquals => write!(f, "-="),
            TokenKind::PlusEquals => write!(f, "+="),
            TokenKind::MultiplyEquals => write!(f, "*="),
            TokenKind::DivideEquals => write!(f, "/="),
            TokenKind::DoubleLessThan => write!(f, "<<"),
            TokenKind::DoubleGreaterThan => write!(f, ">>"),
            TokenKind::QuestionMark => write!(f, "?"),

            // Others
            TokenKind::EOF => write!(f, "EOF"),
            TokenKind::Whitespace => write!(f, "Whitespace"),
            TokenKind::Bad => write!(f, "Bad"),
            TokenKind::Comment => write!(f, "Comment"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Separators
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Colon,
    Semicolon,
    Arrow,
    DoubleDot,
    TripleDot,
    DoubleColon,

    // Literals
    Identifier,
    String(String),
    Float(f64),
    Integer(i64),
    Char(char),

    // Keywords
    Fn,
    Let,
    If,
    Else,
    While,
    For,
    In,
    Return,
    Break,
    Continue,
    Use,
    Pub,
    From,
    Throw,
    Try,
    Catch,
    Loop,
    True,
    False,
    Null,
    Impl,
    Struct,
    Trait,
    Then,
    Const,

    // Operators
    Plus,              // +
    Minus,             // -
    Asterisk,          // *
    Slash,             // /
    Equals,            // =
    Ampersand,         // &
    Pipe,              // |
    Caret,             // ^
    DoubleAsterisk,    // **
    Percent,           // %
    Tilde,             // ~
    GreaterThan,       // >
    LessThan,          // <
    GreaterThanEquals, // >=
    LessThanEquals,    // <=
    EqualsEquals,      // ==
    BangEquals,        // !=
    Bang,              // !
    And,               // &&
    Or,                // ||
    Increment,         // ++
    Decrement,         // --
    MinusEquals,       // -=
    PlusEquals,        // +=
    MultiplyEquals,    // *=
    DivideEquals,      // /=
    DoubleLessThan,    // <<,
    DoubleGreaterThan, // >>,
    QuestionMark,      // ?

    EOF,
    Whitespace,
    Bad,
    Comment,
}

impl TokenKind {
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Fn
                | TokenKind::Let
                | TokenKind::If
                | TokenKind::Else
                | TokenKind::While
                | TokenKind::For
                | TokenKind::In
                | TokenKind::Return
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Use
                | TokenKind::Pub
                | TokenKind::From
                | TokenKind::Throw
                | TokenKind::Try
                | TokenKind::Catch
                | TokenKind::Loop
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Null
                | TokenKind::Impl
                | TokenKind::Struct
                | TokenKind::Trait
                | TokenKind::Then
                | TokenKind::Const
        )
    }
    
    pub fn is_operator(&self) -> bool {
        matches!(
            self,
            TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Asterisk
                | TokenKind::Slash
                | TokenKind::Equals
                | TokenKind::Ampersand
                | TokenKind::Pipe
                | TokenKind::Caret
                | TokenKind::DoubleAsterisk
                | TokenKind::Percent
                | TokenKind::Tilde
                | TokenKind::GreaterThan
                | TokenKind::LessThan
                | TokenKind::GreaterThanEquals
                | TokenKind::LessThanEquals
                | TokenKind::EqualsEquals
                | TokenKind::BangEquals
                | TokenKind::Bang
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::Increment
                | TokenKind::Decrement
                | TokenKind::MinusEquals
                | TokenKind::PlusEquals
                | TokenKind::MultiplyEquals
                | TokenKind::DivideEquals
                | TokenKind::DoubleLessThan
                | TokenKind::DoubleGreaterThan
                | TokenKind::QuestionMark
        )
    }
    
    pub fn is_separator(&self) -> bool {
        matches!(
            self,
            TokenKind::LeftParen
                | TokenKind::RightParen
                | TokenKind::LeftBrace
                | TokenKind::RightBrace
                | TokenKind::LeftBracket
                | TokenKind::RightBracket
                | TokenKind::Comma
                | TokenKind::Dot
                | TokenKind::Colon
                | TokenKind::Semicolon
                | TokenKind::Arrow
                | TokenKind::DoubleDot
                | TokenKind::TripleDot
                | TokenKind::DoubleColon
        )
    }
}