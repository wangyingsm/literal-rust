use black_hawk::handle_request;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = TcpListener::bind("0.0.0.0:8080").await?;
    while let Ok((stream, _)) = server.accept().await {
        tokio::spawn(handle_request(stream));
    }
    Ok(())
}
