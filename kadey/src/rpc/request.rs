use std::net::IpAddr;

use bincode::{Decode, Encode};
use tokio::net::UdpSocket;

use crate::core::node::NodeId;

use super::response::Response;

pub type KeyId = NodeId;

#[derive(Debug, Encode, Decode)]
pub enum Request {
    Ping(NodeId),
    Store {
        src_node_id: NodeId,
        key: Vec<u8>,
        value: Vec<u8>,
    },
    FindNode {
        src_node_id: NodeId,
        node_id: NodeId,
    },
    FindValue {
        src_node_id: NodeId,
        key_id: KeyId,
    },
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
