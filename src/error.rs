use thiserror::Error;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Export error: {0}")]
    Export(String),
    #[error("Fetch error: {0}")]
    Fetch(String),
    #[error("HTTP error: {0}")]
    HttpError(String),
    #[error("Invalid header: {0}")]
    InvalidHeader(String),
    #[error("Invalid line: {0}")]
    InvalidLine(String),
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),
    #[error("Invalid fingerprint: {0}")]
    InvalidFingerprint(String),
    #[error("CSV error: {0}")]
    Csv(String),
}

impl From<std::io::Error> for BridgeError {
    fn from(err: std::io::Error) -> Self {
        BridgeError::Io(err.to_string())
    }
}

impl From<csv::Error> for BridgeError {
    fn from(err: csv::Error) -> Self {
        BridgeError::Export(err.to_string())
    }
}
