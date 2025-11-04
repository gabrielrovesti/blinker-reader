use std::fmt;

#[derive(Debug)]
pub enum BlinkerError {
    Io(std::io::Error),
    Database(String),
    Parsing(String),
    Rendering(String),
    Security(String),
    NotFound(String),
}

impl fmt::Display for BlinkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlinkerError::Io(e) => write!(f, "IO error: {}", e),
            BlinkerError::Database(msg) => write!(f, "Database error: {}", msg),
            BlinkerError::Parsing(msg) => write!(f, "Parsing error: {}", msg),
            BlinkerError::Rendering(msg) => write!(f, "Rendering error: {}", msg),
            BlinkerError::Security(msg) => write!(f, "Security error: {}", msg),
            BlinkerError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for BlinkerError {}

impl From<std::io::Error> for BlinkerError {
    fn from(err: std::io::Error) -> Self {
        BlinkerError::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, BlinkerError>;
