use std::str::FromStr;

use crate::error::RequestParseError;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum ContentType {
    PlainText,
    TextHtml,
    ApplicationJson,
}

impl FromStr for ContentType {
    type Err = RequestParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plain/text" => Ok(Self::PlainText),
            "text/html" => Ok(Self::TextHtml),
            "application/json" => Ok(Self::ApplicationJson),
            _ => Err(RequestParseError::UnknownContentType),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum TransferEncoding {
    Chunked,
}

impl FromStr for TransferEncoding {
    type Err = RequestParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "chunked" => Ok(Self::Chunked),
            _ => Err(RequestParseError::UnknownTransferEncoding),
        }
    }
}
