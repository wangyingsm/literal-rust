use std::net::IpAddr;

use bincode::Encode;
use tokio::net::UdpSocket;

use crate::core::node::NodeId;

use super::response::Response;

pub type KeyId = NodeId;

#[derive(Debug, Encode)]
pub enum Request {
    Ping,
    Store { key: Vec<u8>, value: Vec<u8> },
    FindNode(NodeId),
    FindValue(KeyId),
}

impl Request {
    pub async fn send(&self, peer: IpAddr, port: u16) -> anyhow::Result<Response> {
        let buf = bincode::encode_to_vec(self, bincode::config::standard())?;
        let sock = UdpSocket::bind("0.0.0.0:0").await?;
        sock.connect(format!("{peer}:{port}")).await?;
        sock.send(&buf).await?;
        let mut resp = Vec::with_capacity(4096);
        sock.recv_from(&mut resp).await?;
        let (resp, _) =
            bincode::decode_from_slice::<Response, _>(&resp, bincode::config::standard())?;
        Ok(resp)
    }
}
