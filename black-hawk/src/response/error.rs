use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResponseError {
    #[error("Invalid status code: {0}")]
    InvalidStatusCode(u16),
    #[error("I/O error: {0}")]
    Io(std::io::Error),
    #[error("JSON serialization error: {0}")]
    JsonSerialize(serde_json::Error),
}

impl From<std::io::Error> for ResponseError {
    fn from(err: std::io::Error) -> Self {
        ResponseError::Io(err)
    }
}

impl From<serde_json::Error> for ResponseError {
    fn from(err: serde_json::Error) -> Self {
        ResponseError::JsonSerialize(err)
    }
}
