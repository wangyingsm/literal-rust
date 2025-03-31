use std::path::Path;

use crate::{response::body::IntoBinaryBody, WEB_ROOT};
use async_trait::async_trait;

use crate::{
    request::HttpRequest,
    response::{body::IntoTextBody, error::ResponseError, Response},
};

#[async_trait]
pub trait Handler: Send + Sync {
    async fn handle(&self, request: &HttpRequest) -> Result<Response, ResponseError>;
}

#[derive(Debug)]
pub struct StaticHtmlHandler;

#[async_trait]
impl Handler for StaticHtmlHandler {
    async fn handle(&self, request: &HttpRequest) -> Result<Response, ResponseError> {
        let mut path = String::new();
        path.push_str(WEB_ROOT);
        path.push_str(request.header().path.abs_path());
        println!("{path}");
        let content = tokio::fs::read_to_string(path).await?;
        Response::new(200, "text/html", IntoTextBody::into_body(content))
    }
}

#[derive(Debug)]
pub struct StaticImageHandler;

#[async_trait]
impl Handler for StaticImageHandler {
    async fn handle(&self, request: &HttpRequest) -> Result<Response, ResponseError> {
        let mut path = String::new();
        path.push_str(WEB_ROOT);
        path.push_str(request.header().path.abs_path());
        println!("{path}");
        let p: &Path = request.header().path.abs_path().as_ref();
        let content_type = match p.extension() {
            Some(e) => match e.to_str() {
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpg",
                Some("gif") => "image/gif",
                Some("svg") => "image/svg+xml",
                Some("webp") => "image/webp",
                _ => {
                    return Err(ResponseError::UnknownImageExtension(
                        p.to_string_lossy().to_string(),
                    ))
                }
            },
            None => {
                return Err(ResponseError::UnknownImageExtension(
                    p.to_string_lossy().to_string(),
                ))
            }
        };
        let content = tokio::fs::read(path).await?;
        Response::new(200, content_type, IntoBinaryBody::into_body(content))
    }
}
