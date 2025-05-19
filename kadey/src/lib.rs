use core::{
    node::{Node, NodeId, node_id_distance},
    route::{NodeTuple, NodeTupleWrapper, RouteTableRoot, find_unvisit_nodes},
};
use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc};

use anyhow::bail;
use rpc::{
    request::{KeyId, Request},
    response::{Response, RpcId, ValueResponse},
};
use sha1::Digest;
use tokio::{net::UdpSocket, sync::RwLock, task::JoinSet};
use tracing::{error, level_filters::LevelFilter, warn};

pub mod core;
pub mod rpc;

#[cfg(test)]
pub const ID_BYTES_LENGTH: usize = 4;
#[cfg(not(test))]
pub const ID_BYTES_LENGTH: usize = 20;

pub const K_REPLICATIONS: usize = 20;
pub const ALPHA_PARALLEL: usize = 3;

pub async fn init_app() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();
    dotenv::dotenv()?;
    let seed_node = dotenv::var("SEED_NODES")?;
    let sock_addr = SocketAddr::from_str(&seed_node)?;
    let seed_node_id = dotenv::var("SEED_NODE_ID")?;
    let seed_node_id = hex::decode(seed_node_id)?;
    if seed_node_id.len() != ID_BYTES_LENGTH {
        bail!("invalid seed node id configuration");
    }
    let seed_node_id: NodeId = std::array::from_fn(|i| seed_node_id[i]);
    send_ping((seed_node_id, sock_addr.ip(), sock_addr.port())).await?;
    Ok(())
}

pub async fn handle_request(
    ctx: Arc<AppContext>,
    sock: &UdpSocket,
    req: Vec<u8>,
    remote: SocketAddr,
) {
    let (req, _) = match bincode::decode_from_slice::<Request, _>(&req, bincode::config::standard())
    {
        Ok(r) => r,
        Err(e) => {
            error!("bad request format: {e}");
            return;
        }
    };
    let resp = match req {
        Request::Ping(src_node_id) => ping(ctx, (src_node_id, remote.ip(), remote.port())).await,
        Request::Store {
            src_node_id,
            key,
            value,
        } => store(ctx, key, value, (src_node_id, remote.ip(), remote.port())).await,
        Request::FindNode {
            src_node_id,
            node_id,
        } => find_node(ctx, node_id, (src_node_id, remote.ip(), remote.port())).await,
        Request::FindValue {
            src_node_id,
            key_id,
        } => find_value(ctx, key_id, (src_node_id, remote.ip(), remote.port())).await,
    };
    let resp = match bincode::encode_to_vec(resp, bincode::config::standard()) {
        Ok(r) => r,
        Err(e) => {
            error!("serialize response error: {e}");
            return;
        }
    };
    if let Err(e) = sock.send_to(&resp, remote).await {
        error!("send response to {remote} error: {e}");
    }
}

async fn send_ping(node: NodeTuple) -> anyhow::Result<Response> {
    let req = Request::Ping(node.0);
    let resp = req.send(node.1, node.2).await?;
    if !matches!(resp, Response::Pong { rpc_id: _ }) {
        bail!("invalid response type for ping");
    }
    Ok(resp)
}

#[derive(Debug)]
pub struct AppContext {
    route_table: RwLock<RouteTableRoot>,
    storage: RwLock<HashMap<KeyId, Vec<u8>>>,
    local_node: NodeTuple,
}

impl AppContext {
    pub fn new(node: Node) -> Self {
        Self {
            route_table: RwLock::new(RouteTableRoot::new()),
            storage: RwLock::new(HashMap::new()),
            local_node: (node.node_id(), node.ip_addr(), node.port()),
        }
    }
}

pub async fn find_value(ctx: Arc<AppContext>, key_id: KeyId, peer: NodeTuple) -> Response {
    ctx.route_table.write().await.add_node(peer);
    if let Some(v) = ctx.storage.read().await.get(&key_id) {
        return Response::Value(ValueResponse::Value {
            rpc_id: random_rpc_id(),
            value: v.clone(),
        });
    }
    let mut nearest_k_nodes = ctx.route_table.read().await.find_node(peer.0, key_id).await;
    for ele in nearest_k_nodes.iter_mut() {
        ele.visited = false;
    }
    loop {
        let to_visit = find_unvisit_nodes(&nearest_k_nodes);
        if to_visit.is_empty() {
            warn!("key ({key_id:?}) not found in the network");
            return Response::Value(ValueResponse::NotFound {
                rpc_id: random_rpc_id(),
            });
        }
        let mut join_set = JoinSet::new();
        for i in to_visit {
            let node = nearest_k_nodes[i];
            let ctx = Arc::clone(&ctx);
            join_set.spawn(async move { find_value_remote(ctx, node, key_id).await });
        }
        for result in (join_set.join_all().await).into_iter().flatten() {
            if let ValueResponse::Value { rpc_id: _, value } = result {
                return Response::Value(ValueResponse::Value {
                    rpc_id: random_rpc_id(),
                    value,
                });
            }
        }
    }
}

async fn find_value_remote(
    ctx: Arc<AppContext>,
    node: NodeTupleWrapper,
    key_id: KeyId,
) -> anyhow::Result<ValueResponse> {
    let req = Request::FindValue {
        src_node_id: ctx.local_node.0,
        key_id,
    };
    let resp = req.send(node.node.1, node.node.2).await?;
    match resp {
        Response::Value(value_response) => Ok(value_response),
        _ => bail!("invalid response type for find value"),
    }
}

fn key_id_sha1(key: &[u8]) -> KeyId {
    let mut sha = sha1::Sha1::new();
    sha.update(key);
    let hash = sha.finalize();
    std::array::from_fn(|i| hash[i])
}

pub async fn store(
    ctx: Arc<AppContext>,
    key: Vec<u8>,
    value: Vec<u8>,
    peer: NodeTuple,
) -> Response {
    ctx.route_table.write().await.add_node(peer);
    let key_id = key_id_sha1(&key);
    let mut nearest_k_nodes = ctx
        .route_table
        .read()
        .await
        .find_node(ctx.local_node.0, key_id)
        .await;
    let distance1 = node_id_distance(&ctx.local_node.0, &key_id);
    let distance2 = node_id_distance(&nearest_k_nodes[nearest_k_nodes.len() - 1].node.0, &key_id);
    if distance1 < distance2 {
        ctx.storage.write().await.insert(key_id, value.clone());
    }
    nearest_k_nodes.iter_mut().for_each(|n| n.visited = false);
    loop {
        let to_visit = find_unvisit_nodes(&nearest_k_nodes);
        if to_visit.is_empty() {
            return Response::StoreOk {
                rpc_id: random_rpc_id(),
            };
        }
        let mut join_set = JoinSet::new();
        for i in to_visit {
            let node = nearest_k_nodes[i];
            let key = key.clone();
            let value = value.clone();
            let ctx = Arc::clone(&ctx);
            join_set.spawn(async move { stor_remote(ctx, node, key, value).await });
        }
        join_set.join_all().await;
    }
}

async fn stor_remote(
    ctx: Arc<AppContext>,
    node: NodeTupleWrapper,
    key: Vec<u8>,
    value: Vec<u8>,
) -> anyhow::Result<Response> {
    let req = Request::Store {
        key,
        value,
        src_node_id: ctx.local_node.0,
    };
    let resp = req.send(node.node.1, node.node.2).await?;
    Ok(resp)
}

pub async fn find_node(ctx: Arc<AppContext>, node_id: NodeId, peer: NodeTuple) -> Response {
    ctx.route_table.write().await.add_node(peer);
    let nearest_k_nodes = ctx
        .route_table
        .read()
        .await
        .find_node(ctx.local_node.0, node_id)
        .await;
    let nodes = nearest_k_nodes.into_iter().map(|n| n.node).collect();
    Response::Nodes {
        rpc_id: random_rpc_id(),
        bucket: nodes,
    }
}

pub async fn ping(ctx: Arc<AppContext>, peer: NodeTuple) -> Response {
    ctx.route_table.write().await.add_node(peer);
    Response::Pong {
        rpc_id: random_rpc_id(),
    }
}

fn random_rpc_id() -> RpcId {
    let mut buf = [0; ID_BYTES_LENGTH];
    rand::fill(&mut buf);
    buf
}
