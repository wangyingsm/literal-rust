use thiserror::Error;

#[derive(Debug, Error)]
pub enum RequestParseError {
    #[error("parse http request header error")]
    ParseError,
    #[error("unknown content type")]
    UnknownContentType,
    #[error("unknown transfer encoding")]
    UnknownTransferEncoding,
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for RequestParseError {
    fn from(_value: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        RequestParseError::ParseError
    }
}
