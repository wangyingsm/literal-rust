use std::net::IpAddr;

use tokio::task::JoinSet;

use crate::{
    ALPHA_PARALLEL, K_REPLICATIONS,
    rpc::{request::Request, response::Response},
};

use super::{
    kbucket::KBucket,
    node::{NodeId, get_node_id_bit, node_id_distance},
};

#[derive(Debug)]
pub enum RouteTableEntry {
    Leaf(KBucket),
    Branch {
        zero: Box<RouteTableEntry>,
        one: Box<RouteTableEntry>,
    },
}

pub type NodeTuple = (NodeId, IpAddr, u16);

#[derive(Eq, Clone, Copy)]
pub struct NodeTupleWrapper {
    pub(crate) node: NodeTuple,
    pub(crate) visited: bool,
}

impl PartialEq for NodeTupleWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl RouteTableEntry {
    pub fn new() -> Self {
        Self::Leaf(Default::default())
    }

    pub fn add_node(&mut self, bit: usize, node_id: NodeId, addr: IpAddr, port: u16) {
        match self {
            RouteTableEntry::Leaf(kbucket) => {
                if let super::kbucket::KBucketAddResult::Replaced(zero, one) =
                    kbucket.add_node(node_id, addr, port)
                {
                    *self = RouteTableEntry::Branch {
                        zero: Box::new(RouteTableEntry::Leaf(zero)),
                        one: Box::new(RouteTableEntry::Leaf(one)),
                    };
                }
            }
            RouteTableEntry::Branch { zero, one } => {
                if get_node_id_bit(&node_id, bit) {
                    one.add_node(bit + 1, node_id, addr, port);
                } else {
                    zero.add_node(bit + 1, node_id, addr, port);
                }
            }
        }
    }

    pub fn find_node(&self, node_id: NodeId, bits: usize, found: &mut Vec<NodeTupleWrapper>) {
        match self {
            RouteTableEntry::Leaf(kbucket) => found.extend(
                kbucket
                    .queue()
                    .iter()
                    .map(|n| NodeTupleWrapper {
                        node: (n.node_id(), n.ip_addr(), n.port()),
                        visited: false,
                    })
                    .collect::<Vec<_>>(),
            ),
            RouteTableEntry::Branch { zero, one } => {
                if get_node_id_bit(&node_id, bits) {
                    one.find_node(node_id, bits + 1, found);
                    if found.len() < K_REPLICATIONS {
                        zero.find_node(node_id, bits + 1, found);
                    }
                } else {
                    zero.find_node(node_id, bits + 1, found);
                    if found.len() < K_REPLICATIONS {
                        one.find_node(node_id, bits + 1, found);
                    }
                }
            }
        }
    }
}

impl Default for RouteTableEntry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct RouteTableRoot(RouteTableEntry);

impl RouteTableRoot {
    pub fn new() -> Self {
        Self(RouteTableEntry::new())
    }

    pub fn add_node(&mut self, node: NodeTuple) {
        self.0.add_node(0, node.0, node.1, node.2);
    }

    // find_node, 遍历本节点的路由表，找到K个距离最近的节点，并发地请求alpha个节点，获取node_id最近的节点，
    // 递归结束的条件:所有返回的节点已经不再比目前的K个节点距离更近的时候，结束
    pub async fn find_node(&self, src_node_id: NodeId, node_id: NodeId) -> Vec<NodeTupleWrapper> {
        let mut result = Vec::with_capacity(K_REPLICATIONS);
        self.0.find_node(node_id, 0, &mut result);
        result.sort_by(|lhs, rhs| {
            let distance1 = node_id_distance(&lhs.node.0, &node_id);
            let distance2 = node_id_distance(&rhs.node.0, &node_id);
            distance1.cmp(&distance2)
        });
        // todo!("aysnc request at most alpha(3) remote peers to find node");
        loop {
            let to_visit = find_unvisit_nodes(&result);
            if to_visit.is_empty() {
                break;
            }
            let mut join_set = JoinSet::new();
            {
                for i in to_visit {
                    let p = result[i];
                    result[i].visited = true;
                    join_set.spawn(async move { find_node_remote(src_node_id, node_id, p).await });
                }
            }
            let mut res = join_set.join_all().await;
            while let Some(r) = res.pop() {
                if let Ok(nodes) = r {
                    for node in nodes {
                        let distance = node_id_distance(&node.node.0, &node_id);
                        let index = match result.binary_search_by_key(&distance, |n| {
                            node_id_distance(&n.node.0, &node_id)
                        }) {
                            Ok(index) if index < result.len() - 1 => {
                                if index > 0 {
                                    if result[index - 1] == node || result[index + 1] == node {
                                        continue;
                                    }
                                } else if result[index + 1] == node {
                                    continue;
                                }
                                Some(index)
                            }
                            Err(index) if index < result.len() => Some(index),
                            _ => None,
                        };
                        if let Some(i) = index {
                            result.insert(i, node);
                            result.pop();
                        }
                    }
                }
            }
        }
        result
    }
}

async fn find_node_remote(
    src_node_id: NodeId,
    node_id: NodeId,
    peer: NodeTupleWrapper,
) -> anyhow::Result<Vec<NodeTupleWrapper>> {
    let req = Request::FindNode {
        src_node_id,
        node_id,
    };
    let resp = req.send(peer.node.1, peer.node.2).await?;
    if let Response::Nodes { rpc_id: _, bucket } = resp {
        Ok(bucket
            .into_iter()
            .map(|n| NodeTupleWrapper {
                node: n,
                visited: false,
            })
            .collect())
    } else {
        // Err(anyhow::anyhow!("invalid response type for find_node"))
        anyhow::bail!("invalid response type for find_node")
    }
}

pub(crate) fn find_unvisit_nodes(nodes: &[NodeTupleWrapper]) -> Vec<usize> {
    nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !n.visited)
        .take(ALPHA_PARALLEL)
        .map(|(i, _)| i)
        .collect()
}

impl Default for RouteTableRoot {
    fn default() -> Self {
        Self::new()
    }
}

//
// find_value，先找自己有没有这个key_id，如果有，直接返回value，如果没有，使用find_node找距离key_id最近的
// K个节点，让对方返回value，如果找不到，最终是None。递归结束的条件可能是：其中一个节点直接返回了value。

#[cfg(test)]
mod test {
    use super::*;

    use crate::{
        K_REPLICATIONS,
        core::{kbucket::test::random_ip_addr_and_port, node::Node},
    };

    #[test]
    fn test_route_table_add_node() {
        for _ in 0..100 {
            let mut route_table = RouteTableEntry::new();
            assert!(matches!(route_table, RouteTableEntry::Leaf(_)));
            (0..K_REPLICATIONS).for_each(|i| {
                let (addr, port) = random_ip_addr_and_port();
                let node = Node::from_random_node_id(addr, port);
                route_table.add_node(0, node.node_id(), addr, port);
                assert!(matches!(route_table, RouteTableEntry::Leaf(_)));
                match &route_table {
                    RouteTableEntry::Leaf(b) => assert_eq!(b.len(), i + 1),
                    RouteTableEntry::Branch { .. } => unreachable!(),
                }
            });
            let (addr, port) = random_ip_addr_and_port();
            let node = Node::from_random_node_id(addr, port);
            route_table.add_node(0, node.node_id(), addr, port);
            match &route_table {
                RouteTableEntry::Leaf(_) => unreachable!(),
                RouteTableEntry::Branch { zero, one } => {
                    match &**zero {
                        RouteTableEntry::Leaf(b) => {
                            for node in b.queue().iter() {
                                assert!(!node.get_node_id_bit(0));
                            }
                        }
                        RouteTableEntry::Branch { .. } => unreachable!(),
                    }
                    match &**one {
                        RouteTableEntry::Leaf(b) => {
                            for node in b.queue().iter() {
                                assert!(node.get_node_id_bit(0));
                            }
                        }
                        RouteTableEntry::Branch { .. } => unreachable!(),
                    }
                }
            }
        }
    }
}
