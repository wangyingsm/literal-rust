use std::{io::ErrorKind, path::Path};

use crate::{
    consts::ContentType,
    response::{
        body::{IntoBinaryBody, IntoTextBody},
        status::HttpStatus,
    },
    INDEX_FILE, WEB_ROOT,
};
use async_trait::async_trait;

use crate::{request::HttpRequest, response::Response};

#[async_trait]
pub trait Handler: Send + Sync {
    async fn handle(&self, request: &HttpRequest) -> anyhow::Result<Response>;
}

#[derive(Debug)]
pub struct StaticHandler;

#[async_trait]
impl Handler for StaticHandler {
    async fn handle(&self, request: &HttpRequest) -> anyhow::Result<Response> {
        let mut path = String::new();
        path.push_str(WEB_ROOT);
        path.push_str(request.header().path.abs_path());
        let p: &Path = path.as_ref();
        let ext = p.extension();
        println!("{ext:?}");
        let content_type = match ext {
            Some(e) => ContentType::from_extension(&e.to_string_lossy())?,
            None => {
                if !path.ends_with("/") {
                    path.push('/')
                }
                path.push_str(INDEX_FILE);
                ContentType::TextHtml
            }
        };
        let content = match tokio::fs::read(path).await {
            Ok(c) => c,
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    return Ok(Response::new(
                        HttpStatus::NotFound,
                        &ContentType::PlainText,
                        IntoTextBody::into_body("Not Found!"),
                    ));
                } else {
                    return Err(e.into());
                }
            }
        };
        Ok(Response::new(
            HttpStatus::Ok,
            &content_type,
            IntoBinaryBody::into_body(content),
        ))
    }
}
