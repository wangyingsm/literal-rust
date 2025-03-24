// refactor response module

pub(crate) mod body;
pub mod error;
mod status;

use std::collections::HashMap;

use body::Body;
use error::ResponseError;
use status::HttpStatus;

#[derive(Debug)]
pub struct Response {
    status: HttpStatus,
    headers: HashMap<String, String>,
    body: Body,
}

impl Response {
    pub fn new(status: u16, content_type: &str, body: Body) -> Result<Self, ResponseError> {
        let status = HttpStatus::try_from(status)?;
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), content_type.to_string());
        Ok(Response {
            status,
            headers,
            body,
        })
    }

    pub fn add_header(&mut self, name: impl AsRef<str>, value: impl AsRef<str>) {
        self.headers
            .insert(name.as_ref().to_string(), value.as_ref().to_string());
    }

    pub fn status(&self) -> &HttpStatus {
        &self.status
    }

    pub fn body(&self) -> &Body {
        &self.body
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend(self.status.to_string().as_bytes());
        for (name, value) in &self.headers {
            buffer.extend(format!("{name}: {value}").as_bytes());
        }
        match &self.body {
            Body::RawText(s) => buffer.extend(s.as_bytes()),
            Body::RawBinary(v) => buffer.extend(v),
            Body::Json(j) => buffer.extend(j),
        }
        buffer
    }
}
