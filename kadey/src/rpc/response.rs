use bincode::{Decode, Encode};

use crate::core::route::NodeTuple;

use super::request::KeyId;

pub type RpcId = KeyId;

#[derive(Debug, Decode, Encode)]
pub enum Response {
    Pong {
        rpc_id: RpcId,
    },
    StoreOk {
        rpc_id: RpcId,
    },
    Nodes {
        rpc_id: RpcId,
        bucket: Vec<NodeTuple>,
    },
    Value(ValueResponse),
}

#[derive(Debug, Decode, Encode)]
pub enum ValueResponse {
    Value { rpc_id: RpcId, value: Vec<u8> },
    NotFound { rpc_id: RpcId },
}
