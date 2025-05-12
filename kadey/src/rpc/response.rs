use bincode::Decode;

use crate::core::route::NodeTuple;

use super::request::KeyId;

type RpcId = KeyId;

#[derive(Debug, Decode)]
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
    Value,
}

#[derive(Debug, Decode)]
pub enum ValueResponse {
    Nodes {
        rpc_id: RpcId,
        bucket: Vec<NodeTuple>,
    },
    Value {
        rpc_id: RpcId,
        value: Vec<u8>,
    },
}
