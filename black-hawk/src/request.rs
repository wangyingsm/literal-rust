use std::{collections::HashMap, fmt::Display};

use boundary::Boundary;
use path::HttpPath;
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::{
    consts::{ContentType, TransferEncoding},
    parse::{parse_http_header, parse_request_body},
};

pub mod boundary;
pub mod path;

#[derive(Debug)]
#[allow(unused)]
pub struct HttpRequest {
    pub(crate) header: HttpRequestHeader,
    pub(crate) body: RequestBody,
}

impl HttpRequest {
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
#[allow(unused)]
pub struct HttpRequestHeader {
    pub(crate) method: HttpMethod,
    pub(crate) path: HttpPath,
    pub(crate) version: HttpVersion,
    pub(crate) headers: HttpHeaders,
}

impl HttpRequestHeader {
    pub fn version(&self) -> HttpVersion {
        self.version
    }

    pub fn content_type(&self) -> Option<ContentType> {
        self.headers.0.get("Content-Type")?.parse().ok()
    }

    pub fn transfer_encoding(&self) -> Option<TransferEncoding> {
        self.headers.0.get("Transfer-Encoding")?.parse().ok()
    }

    pub fn content_length(&self) -> Option<usize> {
        self.headers.0.get("Content-Length")?.parse().ok()
    }

    pub fn accept(&self) -> Option<&str> {
        self.headers.0.get("Accept").map(|ac| ac.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Options,
    Delete,
    Patch,
    Connect,
    Head,
    Trace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpVersion {
    V1_0,
    V1_1,
    V2,
    V3,
}

impl Display for HttpVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpVersion::V1_0 => write!(f, "HTTP/1.0"),
            HttpVersion::V1_1 => write!(f, "HTTP/1.1"),
            HttpVersion::V2 => write!(f, "HTTP/2"),
            HttpVersion::V3 => write!(f, "HTTP/3"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestBody {
    Nil,
    RawText(String),
    Chunked { content: String, sizes: Vec<usize> },
    MultiPart(Vec<Boundary>),
    Json(serde_json::Value),
}

pub async fn read_http_request<R: AsyncRead + Unpin>(mut stream: R) -> anyhow::Result<HttpRequest> {
    let mut buf = [0; 4096];
    let mut header_buf = vec![];
    let mut body_buf = vec![];
    loop {
        let n = stream.read(&mut buf).await?;
        if let Some(h_len) = buf[..n].windows(4).position(|w| w == super::DELIMITER) {
            header_buf.extend_from_slice(&buf[..h_len]);
            body_buf.extend_from_slice(&buf[h_len + 4..n]);
            break;
        }
        header_buf.extend_from_slice(&buf[..n]);
    }
    // println!("{}", String::from_utf8_lossy(&header_buf));
    let header = parse_http_header(&header_buf)?;
    let body = if let Some(ContentType::MultiPart(bound)) = header.content_type() {
        let mut boundary = String::new();
        boundary.push_str(bound.as_str());
        boundary.push_str("--\r\n\r\n");
        while !body_buf.ends_with(boundary.as_bytes()) {
            let n = stream.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            body_buf.extend_from_slice(&buf[..n]);
        }
        body_buf
    } else {
        loop {
            let n = stream.read(&mut buf).await?;
            if n == 0 {
                break body_buf;
            }
            if let Some(n) = buf[..n].windows(4).position(|w| w == super::DELIMITER) {
                body_buf.extend_from_slice(&buf[..n]);
                break body_buf;
            }
            body_buf.extend_from_slice(&buf[..n]);
        }
    };

    let body = if body.ends_with(super::DELIMITER) {
        &body[..body.len() - 4]
    } else {
        &body
    };

    let body = if body.is_empty() {
        RequestBody::Nil
    } else {
        parse_request_body(body, &header)?
    };
    Ok(HttpRequest { header, body })
}

#[cfg(test)]
mod test {
    use crate::request::path::Query;

    use super::*;

    #[tokio::test]
    async fn test_http_request_read_firefox_get_with_json() {
        let raw = "GET /favicon.ico HTTP/1.1\r\nHost: 0.0.0.0=5000\r\nContent-Type: application/json\r\nUser-Agent: Mozilla/5.0 (X11; U; Linux i686; en-US; rv:1.9) Gecko/2008061015 Firefox/3.0\r\nAccept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\nAccept-Language: en-us,en;q=0.5\r\nAccept-Encoding: gzip,deflate\r\nAccept-Charset: ISO-8859-1,utf-8;q=0.7,*;q=0.7\r\nKeep-Alive: 300\r\nConnection: keep-alive\r\n\r\n{\"name\":\"hello\",\"age\":42}\r\n\r\n";
        let request = read_http_request(raw.as_bytes()).await.unwrap();
        let body: serde_json::Value =
            serde_json::from_str("{\"name\":\"hello\",\"age\":42}").unwrap();
        assert_eq!(request.body, RequestBody::Json(body));
    }

    #[tokio::test]
    async fn test_http_request_read_post_with_raw_text() {
        let raw = "POST /post_identity_body_world?q=search#hey HTTP/1.1\r\nAccept: */*\r\nContent-Length: 5\r\n\r\nWorld\r\n\r\n";
        let request = read_http_request(raw.as_bytes()).await.unwrap();
        assert_eq!(request.header.method, HttpMethod::Post);
        assert_eq!(request.header.path.abs_path(), "/post_identity_body_world");
        assert_eq!(
            request.header.path.query(),
            vec![Query::Single {
                name: "q".to_string(),
                value: "search".to_string()
            }]
        );
        assert_eq!(request.header.path.anchor(), Some("hey"));
        assert_eq!(request.header.content_length(), Some(5));
        assert_eq!(request.header.accept(), Some("*/*"));
        assert_eq!(request.body, RequestBody::RawText("World".to_string()));
    }

    #[tokio::test]
    async fn test_http_request_read_post_with_chunked() {
        let raw = "POST /post_chunked_all_your_base HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n1e\r\nall your base are belong to us\r\n0\r\n\r\n";
        let request = read_http_request(raw.as_bytes()).await.unwrap();
        assert_eq!(
            request.body,
            RequestBody::Chunked {
                content: "all your base are belong to us".to_string(),
                sizes: vec![0x1e],
            }
        );
    }

    #[tokio::test]
    async fn test_http_request_read_post_with_chunked_triple_zero_ending() {
        let raw = "POST /two_chunks_mult_zero_end HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n6\r\n world\r\n000\r\n\r\n";
        let request = read_http_request(raw.as_bytes()).await.unwrap();
        assert_eq!(
            request.body,
            RequestBody::Chunked {
                content: "hello world".to_string(),
                sizes: vec![5, 6]
            }
        )
    }

    #[tokio::test]
    async fn test_http_request_read_post_with_multipart_formdata() {
        let raw = "POST /upload HTTP/1.1\r\nHost: example.com\r\nContent-Type: multipart/form-data; boundary=--WebKitFormBoundaryABC123\r\nContent-Length: 345\r\n\r\n----WebKitFormBoundaryABC123\r\nContent-Disposition: form-data; name=\"username\"\r\n\r\nJohnDoe\r\n----WebKitFormBoundaryABC123\r\nContent-Disposition: form-data; name=\"file\"; filename=\"example.txt\"\r\nContent-Type: text/plain\r\n\r\nThis is the content of the file.\r\n----WebKitFormBoundaryABC123--\r\n\r\n";
        let request = read_http_request(raw.as_bytes()).await.unwrap();
        assert_eq!(
            request.header.content_type().unwrap(),
            ContentType::MultiPart("--WebKitFormBoundaryABC123".to_string())
        );
        assert_eq!(
            request.body,
            RequestBody::MultiPart(vec![
                Boundary::FormData(Query::Single {
                    name: "username".to_string(),
                    value: "JohnDoe".to_string()
                }),
                Boundary::RawText {
                    name: Some("file".to_string()),
                    filename: "example.txt".to_string(),
                    content: "This is the content of the file.".to_string()
                }
            ])
        )
    }
}
