use std::num::ParseIntError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RequestParseError {
    #[error("parse http request header error")]
    ParseError,
    #[error("unknown content type")]
    UnknownContentType,
    #[error("unknown transfer encoding")]
    UnknownTransferEncoding,
    #[error("empty request path")]
    EmptyRequestPath,
    #[error("parse chunk size error: {0}")]
    ParseChunkSize(ParseIntError),
    #[error("parse chunk content error")]
    ParseChunkContent,
    #[error("chunk content length unmatch, expect: {0}, got: {1}")]
    ChunkContentLengthUnmatch(usize, usize),
    #[error("multipart without boundary set")]
    MultiPartWithoutBoundary,
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for RequestParseError {
    fn from(_value: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        RequestParseError::ParseError
    }
}

impl From<ParseIntError> for RequestParseError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseChunkSize(value)
    }
}
