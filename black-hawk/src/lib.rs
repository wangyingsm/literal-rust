use std::{io::Write, net::TcpStream, time::Duration};

use request::HttpRequest;
use response::{ok_response, Response};

mod error;
pub mod request;
pub mod response;

const DELIMITER: &[u8] = b"\r\n\r\n";

pub fn handle_request(mut stream: TcpStream) {
    let request = match request::read_http_request(&mut stream) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    println!("{request:?}");
    let ok = match request.header.path.as_str() {
        "/" => match handle_root_path(&request) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        },
        _ => todo!("return 404 response"),
    };

    if let Err(e) = stream.write_all(&ok.serialize()) {
        eprintln!("{e}");
    }
}

fn handle_root_path(request: &HttpRequest<Vec<u8>>) -> anyhow::Result<Response<Vec<u8>>> {
    std::thread::sleep(Duration::from_secs(1)); // 模拟数据库之类的操作
    Ok(ok_response(request.header().version()))
}
