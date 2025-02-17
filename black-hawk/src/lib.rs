use request::HttpRequest;
use response::{ok_response, Response};
use tokio::{io::AsyncWriteExt, net::TcpStream};

pub mod consts;
mod error;
pub mod parse;
pub mod request;
pub mod response;

const DELIMITER: &[u8] = b"\r\n\r\n";

pub async fn handle_request(mut stream: TcpStream) {
    let request = match request::read_http_request(&mut stream).await {
        Ok(req) => req,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    // println!("{request:?}");
    let ok = match request.header.path.abs_path() {
        "/" => match handle_root_path(&request).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        },
        _ => todo!("return 404 response"),
    };

    if let Err(e) = stream.write_all(&ok.serialize()).await {
        eprintln!("{e}");
    }
}

async fn handle_root_path(request: &HttpRequest) -> anyhow::Result<Response<Vec<u8>>> {
    // let _ = reqwest::get("https://www.rust-lang.org").await?;
    // sleep(Duration::from_secs(1)).await;
    Ok(ok_response(request.header().version()))
}
