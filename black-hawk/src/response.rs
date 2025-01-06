use std::collections::HashMap;

use crate::request::{HttpHeaders, HttpVersion};

#[derive(Debug)]
pub struct Response<B> {
    version: HttpVersion,
    status_code: u16,
    status: String,
    header: HttpHeaders,
    body: Option<B>,
}

pub fn ok_response(version: HttpVersion) -> Response<Vec<u8>> {
    Response {
        version,
        status_code: 200,
        status: "OK".to_string(),
        header: HttpHeaders(HashMap::new()),
        body: None,
    }
}

impl<B> Response<B> {
    pub fn serialize(&self) -> Vec<u8>
    where
        B: serde::Serialize,
    {
        let mut result = vec![];
        result.extend(
            format!("{} {} {}\r\n", self.version, self.status_code, self.status).as_bytes(),
        );
        for (k, v) in self.header.0.iter() {
            result.extend(format!("{}: {}", k, v).as_bytes());
        }
        result
    }
}
