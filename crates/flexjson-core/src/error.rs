use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Error, Serialize)]
pub enum ParseError {
    #[error("E_UNMATCHED_BRACKET: {0}")]
    UnmatchedBracket(String),
    #[error("E_UNEXPECTED_TOKEN: {0}")]
    UnexpectedToken(String),
    #[error("E_INVALID_ESCAPE: {0}")]
    InvalidEscape(String),
    #[error("E_DUPLICATE_KEY: {0}")]
    DuplicateKey(String),
    #[error("E_MAX_FRAGMENT_SIZE: {0}")]
    MaxFragmentSize(String),
    #[error("E_ENCODING: {0}")]
    Encoding(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct ParseWarning {
    pub fragment_index: usize,
    pub offset: usize,
    pub line: usize,
    pub col: usize,
    pub code: String,
    pub message: String,
}
