use std::str::FromStr;

use nom::bytes::complete::tag;

use crate::error::RequestParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContentType {
    PlainText,
    TextHtml,
    TextCss,
    TextJavascript,
    ImagePng,
    ImageJpeg,
    ImageGif,
    ImageWebp,
    ImageSvg,
    VideoMp4,
    VideoWebm,
    AudioMp3,
    ApplicationJson,
    MultiPart(String),
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PlainText => "plain/text",
            Self::TextHtml => "text/html",
            Self::TextCss => "text/css",
            Self::TextJavascript => "text/javascript",
            Self::ImagePng => "image/png",
            Self::ImageJpeg => "image/jpeg",
            Self::ImageGif => "image/gif",
            Self::ImageWebp => "image/webp",
            Self::ImageSvg => "image/svg+xml",
            Self::VideoMp4 => "video/mp4",
            Self::VideoWebm => "video/webm",
            Self::AudioMp3 => "audio/mp3",
            Self::ApplicationJson => "application/json",
            Self::MultiPart(_) => "multipart/form-data",
            // _ => "application/octet-stream",
        }
    }

    pub fn from_extension(extension: &str) -> Result<Self, RequestParseError> {
        match extension {
            "txt" => Ok(Self::PlainText),
            "html" => Ok(Self::TextHtml),
            "css" => Ok(Self::TextCss),
            "js" => Ok(Self::TextJavascript),
            "png" => Ok(Self::ImagePng),
            "jpg" | "jpeg" => Ok(Self::ImageJpeg),
            "gif" => Ok(Self::ImageGif),
            "webp" => Ok(Self::ImageWebp),
            "svg" => Ok(Self::ImageSvg),
            "mp4" => Ok(Self::VideoMp4),
            "webm" => Ok(Self::VideoWebm),
            "mp3" => Ok(Self::AudioMp3),
            "json" => Ok(Self::ApplicationJson),
            _ => Err(RequestParseError::UnknownContentType),
        }
    }
}

impl FromStr for ContentType {
    type Err = RequestParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plain/text" => Ok(Self::PlainText),
            "text/html" => Ok(Self::TextHtml),
            "text/css" => Ok(Self::TextCss),
            "text/javascript" | "application/javascript" => Ok(Self::TextJavascript),
            "image/png" => Ok(Self::ImagePng),
            "image/jpeg" | "image/jpg" => Ok(Self::ImageJpeg),
            "image/gif" => Ok(Self::ImageGif),
            "image/webp" => Ok(Self::ImageWebp),
            "image/svg+xml" => Ok(Self::ImageSvg),
            "video/mp4" => Ok(Self::VideoMp4),
            "video/webm" => Ok(Self::VideoWebm),
            "audio/mp3" => Ok(Self::AudioMp3),
            "application/json" => Ok(Self::ApplicationJson),
            s if s.starts_with("multipart/form-data") => {
                let (i, _) = tag("multipart/form-data; boundary=")(s.as_bytes())?;
                Ok(Self::MultiPart(String::from_utf8_lossy(i).to_string()))
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
