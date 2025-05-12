use std::{array, net::IpAddr, time::SystemTime};

use sha1::{Digest, Sha1};

use crate::ID_BYTES_LENGTH;

pub type NodeId = [u8; ID_BYTES_LENGTH];

#[derive(Debug, Clone, Copy, Eq)]
pub struct Node {
    node_id: NodeId,
    ip_addr: IpAddr,
    port: u16,
    last_seen: SystemTime,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
    }
}

impl Node {
    pub fn new(node_id: NodeId, ip_addr: IpAddr, port: u16) -> Self {
        Node {
            node_id,
            ip_addr,
            port,
            last_seen: SystemTime::now(),
        }
    }

    pub fn from_random_node_id(ip_addr: IpAddr, port: u16) -> Self {
        let mut node_id = [0; ID_BYTES_LENGTH];
        rand::fill(&mut node_id);
        Node::new(node_id, ip_addr, port)
    }

    pub fn from_ip_addr_sha1(ip_addr: IpAddr, port: u16) -> Self {
        let mut sha1 = Sha1::new();
        match ip_addr {
            IpAddr::V4(a) => sha1.update(a.octets()),
            IpAddr::V6(a) => sha1.update(a.octets()),
        }
        sha1.update(port.to_le_bytes());
        let hash = sha1.finalize();
        let node_id = core::array::from_fn(|i| hash[i]);
        Node::new(node_id, ip_addr, port)
    }

    pub(crate) fn get_node_id_bit(&self, i: usize) -> bool {
        get_node_id_bit(&self.node_id, i)
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = SystemTime::now();
    }

    pub fn ip_addr(&self) -> IpAddr {
        self.ip_addr
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

pub(crate) fn get_node_id_bit(node_id: &NodeId, i: usize) -> bool {
    let byte_index = i / 8;
    let bit_index = i % 8;
    node_id[byte_index] & (1 << (8 - bit_index - 1)) != 0
}

pub(crate) fn node_id_distance(lhs: &NodeId, rhs: &NodeId) -> NodeId {
    array::from_fn(|i| lhs[i] ^ rhs[i])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_node_id_bit() {
        let node_id = [0b1111_0000, 0b01011010, 0b10001110, 0b10010110];
        assert!(get_node_id_bit(&node_id, 0));
        assert!(get_node_id_bit(&node_id, 1));
        assert!(get_node_id_bit(&node_id, 2));
        assert!(get_node_id_bit(&node_id, 3));
        assert!(!get_node_id_bit(&node_id, 4));
        assert!(!get_node_id_bit(&node_id, 5));
        assert!(!get_node_id_bit(&node_id, 6));
        assert!(!get_node_id_bit(&node_id, 7));

        assert!(!get_node_id_bit(&node_id, 8));
        assert!(get_node_id_bit(&node_id, 9));
        assert!(!get_node_id_bit(&node_id, 10));
        assert!(get_node_id_bit(&node_id, 11));
        assert!(get_node_id_bit(&node_id, 12));
        assert!(!get_node_id_bit(&node_id, 13));
        assert!(get_node_id_bit(&node_id, 14));
        assert!(!get_node_id_bit(&node_id, 15));

        assert!(get_node_id_bit(&node_id, 16));
        assert!(!get_node_id_bit(&node_id, 17));
        assert!(!get_node_id_bit(&node_id, 18));
        assert!(!get_node_id_bit(&node_id, 19));
        assert!(get_node_id_bit(&node_id, 20));
        assert!(get_node_id_bit(&node_id, 21));
        assert!(get_node_id_bit(&node_id, 22));
        assert!(!get_node_id_bit(&node_id, 23));

        assert!(get_node_id_bit(&node_id, 24));
        assert!(!get_node_id_bit(&node_id, 25));
        assert!(!get_node_id_bit(&node_id, 26));
        assert!(get_node_id_bit(&node_id, 27));
        assert!(!get_node_id_bit(&node_id, 28));
        assert!(get_node_id_bit(&node_id, 29));
        assert!(get_node_id_bit(&node_id, 30));
        assert!(!get_node_id_bit(&node_id, 31));
    }
}
