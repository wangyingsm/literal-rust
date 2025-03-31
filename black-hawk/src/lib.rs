use std::sync::Arc;

use handler::{StaticHtmlHandler, StaticImageHandler};
use request::HttpMethod;
use response::Response;
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

pub struct AppContext {
    static_router: StaticRouter,
}

impl AppContext {
    pub fn new() -> Self {
        let mut static_router = StaticRouter::new();
        static_router.add_route(
            &HttpMethod::Get,
            "/static/index.html".to_string(),
            Box::new(StaticHtmlHandler),
        );
        static_router.add_route(
            &HttpMethod::Get,
            "/static/images/logo.png".to_string(),
            Box::new(StaticImageHandler),
        );
        // static_router.add_route(
        //     &HttpMethod::Get,
        //     "/static/images/*".to_string(),
        //     Box::new(StaticImageHandler),
        // );
        // static_router.add_route(
        //     &HttpMethod::Get,
        //     "/api/user/{user_id}".to_string(),
        //     Box::new(StaticHtmlHandler),
        // );
        Self { static_router }
    }
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn handle_request(mut stream: TcpStream, context: Arc<AppContext>) {
    let request = match request::read_http_request(&mut stream).await {
        Ok(req) => req,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    let abs_path = request.header().path.abs_path();
    let response = match &abs_path.as_bytes()[..8] {
        b"/static/" => match context.static_router.route(&request) {
            Some(handler) => match handler.handle(&request).await {
                Ok(r) => r,
                Err(e) => Response::new(
                    500,
                    "text/plain",
                    response::body::Body::RawText(format!("Internal Server Error: {e}")),
                )
                .unwrap(),
            },
            None => Response::new(
                404,
                "text/plain",
                response::body::Body::RawText("Not Found".to_string()),
            )
            .unwrap(),
        },
        _ => Response::new(
            404,
            "text/plain",
            response::body::Body::RawText("Not Found".to_string()),
        )
        .unwrap(),
    };
    println!("{response:?}");

    if let Err(e) = stream.write_all(&response.serialize()).await {
        eprintln!("{e}");
    }
}
