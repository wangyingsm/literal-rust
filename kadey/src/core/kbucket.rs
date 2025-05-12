use std::{collections::VecDeque, net::IpAddr};

use bit_vec::BitVec;

use crate::K_REPLICATIONS;

use super::node::{Node, NodeId, get_node_id_bit};

#[derive(Debug, PartialEq, Eq)]
pub enum KBucketAddResult {
    Added,
    Replaced(KBucket, KBucket),
    Updated,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct KBucket {
    prefix_bits: BitVec,
    queue: VecDeque<Node>,
}

impl KBucket {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.len() >= K_REPLICATIONS
    }

    pub fn add_node(&mut self, node_id: NodeId, ip_addr: IpAddr, port: u16) -> KBucketAddResult {
        let pos = self.queue.iter().position(|n| n.node_id() == node_id);
        if let Some(p) = pos {
            self.queue[p].update_last_seen();
            let node = self.queue.drain(p..p + 1).next().unwrap();
            self.queue.push_back(node);
            return KBucketAddResult::Updated;
        }
        let node = Node::new(node_id, ip_addr, port);
        if self.is_full() {
            if self.prefix_bits.len() < K_REPLICATIONS * 8 {
                let (zero, one) = self.split_k_buckets(node_id, ip_addr, port);
                return KBucketAddResult::Replaced(zero, one);
            } else {
                todo!("ping RPC to the least recently last_seen node");
                // if no pong response returned, remove the node from the queue
                // self.queue.pop_front();
                // self.queue.push_back(node);
            }
            // return;
        }
        self.queue.push_back(node);
        KBucketAddResult::Added
    }

    pub fn split_k_buckets(&mut self, node_id: NodeId, ip_addr: IpAddr, port: u16) -> (Self, Self) {
        let mut zero_prefix_bits = self.prefix_bits.clone();
        let mut one_prefix_bits = self.prefix_bits.clone();
        zero_prefix_bits.push(false);
        one_prefix_bits.push(true);
        let mut zero_queue = VecDeque::new();
        let mut one_queue = VecDeque::new();
        while let Some(node) = self.queue.pop_front() {
            if node.get_node_id_bit(self.prefix_bits.len()) {
                one_queue.push_back(node);
            } else {
                zero_queue.push_back(node);
            }
        }
        let (mut zero_branch, mut one_branch) = (
            Self {
                prefix_bits: zero_prefix_bits,
                queue: zero_queue,
            },
            Self {
                prefix_bits: one_prefix_bits,
                queue: one_queue,
            },
        );
        if get_node_id_bit(&node_id, self.prefix_bits.len()) {
            one_branch.add_node(node_id, ip_addr, port);
        } else {
            zero_branch.add_node(node_id, ip_addr, port);
        }
        (zero_branch, one_branch)
    }

    pub fn queue(&self) -> &VecDeque<Node> {
        &self.queue
    }
}

#[cfg(test)]
pub(crate) mod test {
    use std::str::FromStr;

    use super::*;

    pub(crate) fn random_ip_addr_and_port() -> (IpAddr, u16) {
        let mut addr = [0; 4];
        rand::fill(&mut addr);
        (IpAddr::V4(addr.into()), rand::random_range(1024..=65535))
    }

    #[test]
    fn test_split_k_buckets() {
        for _ in 0..100 {
            let mut k_bucket = KBucket::default();
            (0..K_REPLICATIONS).for_each(|_| {
                let (addr, port) = random_ip_addr_and_port();
                let node = Node::from_random_node_id(addr, port);
                k_bucket.add_node(node.node_id(), addr, port);
            });
            let (addr, port) = random_ip_addr_and_port();
            let node = Node::from_random_node_id(addr, port);
            let (zero_branch, one_branch) = k_bucket.split_k_buckets(node.node_id(), addr, port);
            for node in zero_branch.queue.iter() {
                assert!(!node.get_node_id_bit(0));
            }
            for node in one_branch.queue.iter() {
                assert!(node.get_node_id_bit(0));
            }
            if node.get_node_id_bit(0) {
                assert!(one_branch.queue.contains(&node));
            } else {
                assert!(zero_branch.queue.contains(&node));
            }
        }
    }

    #[test]
    fn test_add_existed_node_to_k_bucket() {
        let mut bucket = KBucket::new();
        let node_id = [0, 1, 2, 3];
        let ip_addr = IpAddr::from_str("127.0.0.1").unwrap();
        let port = 65535;
        let result = bucket.add_node(node_id, ip_addr, port);
        assert_eq!(result, KBucketAddResult::Added);
        let ip_addr = IpAddr::from_str("192.168.1.1").unwrap();
        let port = 1025;
        let result = bucket.add_node(node_id, ip_addr, port);
        assert_eq!(result, KBucketAddResult::Updated);
        assert_eq!(bucket.len(), 1);
        let node_id = [4, 3, 2, 1];
        let result = bucket.add_node(node_id, ip_addr, port);
        assert_eq!(result, KBucketAddResult::Added);
        assert_eq!(bucket.len(), 2);
    }
}
