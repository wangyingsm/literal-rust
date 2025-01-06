use std::net::TcpListener;

use black_hawk::handle_request;

fn main() -> anyhow::Result<()> {
    let server = TcpListener::bind("0.0.0.0:8080")?;
    for stream in server.incoming() {
        let Ok(stream) = stream else {
            continue;
        };
        std::thread::spawn(move || handle_request(stream));
    }

    Ok(())
}
