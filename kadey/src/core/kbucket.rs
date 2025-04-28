use std::{collections::VecDeque, net::IpAddr};

use bit_vec::BitVec;

use crate::K_REPLICATIONS;

use super::node::{Node, NodeId, get_node_id_bit};

#[derive(Debug)]
pub enum KBucketAddResult {
    Added,
    Replaced(KBucket, KBucket),
}

#[derive(Debug, Default)]
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

    pub fn is_full(&self) -> bool {
        self.len() >= K_REPLICATIONS
    }

    pub fn add_node(&mut self, node_id: NodeId, ip_addr: IpAddr, port: u16) -> KBucketAddResult {
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
}
