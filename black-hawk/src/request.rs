use std::{collections::HashMap, fmt::Display, io::Read, net::TcpStream};

use nom::bytes::complete::{tag, take_until};

use crate::error::RequestError;

#[derive(Debug)]
pub struct HttpRequest<B> {
    pub(crate) header: HttpRequestHeader,
    pub(crate) body: Option<B>,
}

impl<B> HttpRequest<B> {
    pub fn header(&self) -> &HttpRequestHeader {
        &self.header
    }
}

#[derive(Debug)]
pub struct HttpHeaders(pub(crate) HashMap<String, String>);

impl From<HashMap<String, String>> for HttpHeaders {
    fn from(value: HashMap<String, String>) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct HttpRequestHeader {
    pub(crate) method: HttpMethod,
    pub(crate) path: String,
    pub(crate) version: HttpVersion,
    pub(crate) headers: HttpHeaders,
}

impl HttpRequestHeader {
    pub fn version(&self) -> HttpVersion {
        self.version
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum HttpMethod {
    Get,
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum HttpVersion {
    V1_0,
    V1_1,
    V2,
}

impl Display for HttpVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpVersion::V1_0 => write!(f, "HTTP/1.0"),
            HttpVersion::V1_1 => write!(f, "HTTP/1.1"),
            HttpVersion::V2 => write!(f, "HTTP/2"),
        }
    }
}

pub fn read_http_request(stream: &mut TcpStream) -> anyhow::Result<HttpRequest<Vec<u8>>> {
    let mut buf = [0; 4096];
    let mut header_buf = vec![];
    loop {
        let n = stream.read(&mut buf)?;
        if let Some(n) = buf[..n].windows(4).position(|w| w == super::DELIMITER) {
            header_buf.extend_from_slice(&buf[..n]);
            break;
        }
        header_buf.extend_from_slice(&buf[..n]);
    }
    println!("{}", String::from_utf8_lossy(&header_buf));
    let header = parse_header(&header_buf)?;
    Ok(HttpRequest { header, body: None })
}

fn parse_header(i: &[u8]) -> anyhow::Result<HttpRequestHeader> {
    let (i, method) = parse_request_method(i).map_err(|_| RequestError::ParseHeaderError)?;
    let (i, path) = take_until::<_, _, nom::error::Error<&[u8]>>(" ")(i)
        .map_err(|_| RequestError::ParseHeaderError)?;
    let (i, _) = tag::<_, _, nom::error::Error<&[u8]>>(b" ")(i)
        .map_err(|_| RequestError::ParseHeaderError)?;
    let path = String::from_utf8_lossy(path).to_string();
    let (i, _) = tag::<_, _, nom::error::Error<&[u8]>>(b"HTTP/")(i)
        .map_err(|_| RequestError::ParseHeaderError)?;
    let (i, version_bytes) = take_until_new_line(i)?;
    let version = match version_bytes {
        b"1.0" => HttpVersion::V1_0,
        b"1.1" => HttpVersion::V1_1,
        b"2" => HttpVersion::V2,
        _ => unimplemented!("version not support yet"),
    };
    let i = consume_newline(i)?;
    let mut headers = HashMap::new();
    let header_lines = String::from_utf8_lossy(i);
    for l in header_lines.lines() {
        let mut splits = l.split(": ");
        let name = splits.next().ok_or(RequestError::ParseHeaderError)?;
        let value = splits.next().ok_or(RequestError::ParseHeaderError)?;
        headers.insert(name.to_string(), value.to_string());
    }
    Ok(HttpRequestHeader {
        method,
        path,
        version,
        headers: headers.into(),
    })
}

fn take_until_new_line(i: &[u8]) -> anyhow::Result<(&[u8], &[u8])> {
    Ok(take_until::<_, _, nom::error::Error<&[u8]>>("\r\n")(i)
        .map_err(|_| RequestError::ParseHeaderError)?)
}

fn consume_newline(i: &[u8]) -> anyhow::Result<&[u8]> {
    let (i, _) = tag::<_, _, nom::error::Error<&[u8]>>(b"\r\n")(i)
        .map_err(|_| RequestError::SkipNewlineError)?;
    Ok(i)
}

fn parse_request_method(i: &[u8]) -> anyhow::Result<(&[u8], HttpMethod)> {
    let (i, _) = tag::<_, _, nom::error::Error<&[u8]>>("GET ")(i)
        .map_err(|_| RequestError::ParseHeaderError)?;
    Ok((i, HttpMethod::Get))
}
