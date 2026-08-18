use thiserror::Error;

#[derive(Error, Debug)]
pub enum DagrError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database / Storage Error: {0}")]
    Storage(String),

    #[error("Parser Initialization Error: {0}")]
    ParserInit(String),

    #[error("Parse Failure: {0}")]
    ParseFailure(String),

    #[error("Unsupported Language: {0}")]
    UnsupportedLanguage(String),

    #[error("Symbol Not Found: '{symbol}' in file '{file}'")]
    SymbolNotFound { symbol: String, file: String },

    #[error("Sandbox Execution Error: {0}")]
    Sandbox(String),

    #[error("Configuration Error: {0}")]
    Config(String),

    #[error("Rule Violation: {0}")]
    RuleViolation(String),

    #[error("Serialization Error: {0}")]
    Serialization(String),

    #[error("Tokenizer Error: {0}")]
    Tokenizer(String),
}

pub type Result<T> = std::result::Result<T, DagrError>;

impl From<rusqlite::Error> for DagrError {
    fn from(err: rusqlite::Error) -> Self {
        DagrError::Storage(err.to_string())
    }
}

impl From<serde_json::Error> for DagrError {
    fn from(err: serde_json::Error) -> Self {
        DagrError::Serialization(err.to_string())
    }
}
