use std::str::FromStr;

use crate::error::RequestParseError;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ContentType {
    PlainText,
    TextHtml,
    ApplicationJson,
    MultiPart(String),
}

impl FromStr for ContentType {
    type Err = RequestParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plain/text" => Ok(Self::PlainText),
            "text/html" => Ok(Self::TextHtml),
            "application/json" => Ok(Self::ApplicationJson),
            s if s.starts_with("multipart/form-data") => {
                let pos = s
                    .chars()
                    .position(|x| x == '=')
                    .ok_or(RequestParseError::MultiPartWithoutBoundary)?;
                Ok(Self::MultiPart(s[pos + 1..].to_string()))
            }
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
