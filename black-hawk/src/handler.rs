use std::path::PathBuf;

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
        const WEB_ROOT: &str = "/html";
        let mut path = PathBuf::new();
        path.push(WEB_ROOT);
        path.push(request.header().path.abs_path());
        let content = tokio::fs::read_to_string(path).await?;
        Response::new(200, "text/html", IntoTextBody::into_body(content))
    }
}
