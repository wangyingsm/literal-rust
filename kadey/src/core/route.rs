use std::net::IpAddr;

use super::{
    kbucket::KBucket,
    node::{NodeId, get_node_id_bit},
};

#[derive(Debug)]
pub enum RouteTableEntry {
    Leaf(KBucket),
    Branch {
        zero: Box<RouteTableEntry>,
        one: Box<RouteTableEntry>,
    },
}

impl RouteTableEntry {
    pub fn new() -> Self {
        Self::Leaf(Default::default())
    }

    pub fn add_node(&mut self, bit: usize, node_id: NodeId, addr: IpAddr, port: u16) {
        match self {
            RouteTableEntry::Leaf(kbucket) => match kbucket.add_node(node_id, addr, port) {
                super::kbucket::KBucketAddResult::Added => (),
                super::kbucket::KBucketAddResult::Replaced(zero, one) => {
                    *self = RouteTableEntry::Branch {
                        zero: Box::new(RouteTableEntry::Leaf(zero)),
                        one: Box::new(RouteTableEntry::Leaf(one)),
                    };
                }
            },
            RouteTableEntry::Branch { zero, one } => {
                if get_node_id_bit(&node_id, bit) {
                    one.add_node(bit + 1, node_id, addr, port);
                } else {
                    zero.add_node(bit + 1, node_id, addr, port);
                }
            }
        }
    }
}

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
