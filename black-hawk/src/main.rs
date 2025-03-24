use std::sync::Arc;

use black_hawk::{handle_request, AppContext};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let context = Arc::new(AppContext::new());
    let server = TcpListener::bind("0.0.0.0:8080").await?;
    while let Ok((stream, _)) = server.accept().await {
        tokio::spawn(handle_request(stream, Arc::clone(&context)));
    }
    Ok(())
}
