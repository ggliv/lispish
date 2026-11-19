use crate::lexer::Token;

pub type Result<T> = std::result::Result<T, LispishError>;

// TODO add index/range into original source code for easier error locating
#[derive(Debug, PartialEq, Clone)]
pub enum LispishError {
    InvalidEscapeSequence(char),
    InvalidNumber(String),
    UnterminatedString(String),
    UnexpectedEof,
    UnexpectedToken(Token),
    EmptyApplication,
    ArityMismatch {
        name: String,
        expected: usize,
        found: usize,
    },
    TypeMismatch,
    UnboundSymbol(String),
    EmptyList,
}

impl std::fmt::Display for LispishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LispishError::InvalidEscapeSequence(c) => write!(f, "Invalid character escape sequence: \\{c}"),
            LispishError::InvalidNumber(n) => write!(f, "Invalid number literal: {n}"),
            LispishError::UnterminatedString(s) => write!(f, "Unterminated string literal: {s}"),
            LispishError::UnexpectedEof => write!(f, "Unexpected end of input"),
            LispishError::UnexpectedToken(t) => write!(f, "Unexpected token: {t:?}"),
            LispishError::EmptyApplication => write!(f, "Cannot evaluate empty list ()"),
            LispishError::ArityMismatch {
                name,
                expected,
                found,
            } => write!(f, "`{name}` expected {expected} arguments, got {found}"),
            LispishError::TypeMismatch => write!(f, "Type mismatch"),
            LispishError::UnboundSymbol(s) => write!(f, "Unbound symbol {s}"),
            LispishError::EmptyList => write!(f, "Empty list"),
        }
    }
}

impl std::error::Error for LispishError {}
