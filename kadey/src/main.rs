use std::{net::SocketAddr, str::FromStr, sync::Arc};

use kadey::{AppContext, core::node::Node, handle_request, init_app};
use tokio::net::UdpSocket;
use tracing::error;

// TODO:
// 1. 引入clap实现命令行参数解析和初始化过程
// 2. 实现unix socket，侦听来自命令行的指令，比方: kadey start -c config_file, kadey stop, kadey find-value xxx
// 3. 实现JSON RPC，提供第三方应用的调用
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_app().await?;
    let local_addr = dotenv::var("LOCAL_NODE")?;
    let sock = UdpSocket::bind(&local_addr).await?;
    let local_addr = SocketAddr::from_str(&local_addr)?;
    let local_node = Node::from_random_node_id(local_addr.ip(), local_addr.port());
    let ctx = Arc::new(AppContext::new(local_node));
    loop {
        let mut buf = [0; 65536];
        let ctx = Arc::clone(&ctx);
        match sock.recv_from(&mut buf).await {
            Ok((n, remote)) => {
                handle_request(ctx, &sock, buf[..n].to_vec(), remote).await;
            }
            Err(e) => {
                error!("receive from peer error: {e}");
            }
        }
    }
}
