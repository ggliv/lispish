pub type Result<T> = std::result::Result<T, LispishError>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LispishError {
    InvalidNumber(String),
    UnterminatedString(String),
}
