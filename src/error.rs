use crate::lexer::Token;

pub type Result<T> = std::result::Result<T, LispishError>;

// TODO add index/range into original source code for easier error locating
#[derive(Debug, PartialEq, Clone)]
pub enum LispishError {
    InvalidNumber(String),
    UnterminatedString(String),
    UnexpectedEof,
    UnexpectedToken(Token),
}

impl std::fmt::Display for LispishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LispishError::InvalidNumber(n) => write!(f, "Invalid number literal: {n}"),
            LispishError::UnterminatedString(s) => write!(f, "Unterminated string literal: {s}"),
            LispishError::UnexpectedEof => write!(f, "Unexpected end of input"),
            LispishError::UnexpectedToken(t) => write!(f, "Unexpected token: {t:?}"),
        }
    }
}

impl std::error::Error for LispishError {}
