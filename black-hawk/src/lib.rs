use std::sync::Arc;

use consts::ContentType;
use handler::StaticHandler;
use request::HttpMethod;
use response::{status::HttpStatus, Response};
use route::{Router, StaticRouter};
use tokio::{io::AsyncWriteExt, net::TcpStream};

pub mod consts;
mod error;
mod handler;
pub mod parse;
pub mod request;
pub mod response;
pub mod route;

const DELIMITER: &[u8] = b"\r\n\r\n";
pub const WEB_ROOT: &str = "html";
pub const INDEX_FILE: &str = "index.html";

pub struct AppContext {
    static_router: StaticRouter,
}

// 'static并不代表生命周期是完全静态的，代表修饰的value能够在程序运行的整个生命周期中存在
impl AppContext {
    pub fn new() -> Self {
        let mut static_router = StaticRouter::new();
        // /static/images/logo.png should be handled by StaticHandler with path /images/logo.png
        static_router.add_route(&HttpMethod::Get, "/*", Box::new(StaticHandler));
        // /api/user/{user_id} should match any /user/12345 like path and
        // put `12345` into path variable `user_id`
        Self { static_router }
    }
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn handle_request(mut stream: TcpStream, context: Arc<AppContext>) {
    let mut request = match request::read_http_request(&mut stream).await {
        Ok(req) => req,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    let abs_path = request.header().path.abs_path();
    println!("abs_path: {abs_path}");
    let response = if abs_path.starts_with("/static/") {
        *request.header_mut().path.abs_path_mut() = abs_path[7..].to_string();
        match context.static_router.route(&mut request) {
            Some(handler) => match handler.handle(&request).await {
                Ok(r) => r,
                Err(e) => Response::new(
                    HttpStatus::InternalServerError,
                    &ContentType::PlainText,
                    response::body::Body::RawText(format!("Internal Server Error: {e}")),
                ),
            },
            None => Response::new(
                HttpStatus::NotFound,
                &ContentType::PlainText,
                response::body::Body::RawText("Not Found".to_string()),
            ),
        }
    } else {
        Response::new(
            HttpStatus::NotFound,
            &ContentType::PlainText,
            response::body::Body::RawText("Not Found".to_string()),
        )
    };
    println!("{response:?}");

    if let Err(e) = stream.write_all(&response.serialize()).await {
        eprintln!("{e}");
    }
}
