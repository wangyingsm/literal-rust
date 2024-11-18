use std::ops::{AddAssign, BitAnd, BitOrAssign, Index, Shl, Sub};

pub type TrieHarderMapU8<'th, V> = TrieHarderMap<'th, u8, V>;
pub type TrieHarderMapU16<'th, V> = TrieHarderMap<'th, u16, V>;
pub type TrieHarderMapU32<'th, V> = TrieHarderMap<'th, u32, V>;
pub type TrieHarderMapU64<'th, V> = TrieHarderMap<'th, u64, V>;
pub type TrieHarderMapU128<'th, V> = TrieHarderMap<'th, u128, V>;

pub type TrieHarderSetU8<'th> = TrieHarderSet<'th, u8>;
pub type TrieHarderSetU16<'th> = TrieHarderSet<'th, u16>;
pub type TrieHarderSetU32<'th> = TrieHarderSet<'th, u32>;
pub type TrieHarderSetU64<'th> = TrieHarderSet<'th, u64>;
pub type TrieHarderSetU128<'th> = TrieHarderSet<'th, u128>;

pub type TrieHarderSet<'th, T> = TrieHarderMap<'th, T, ()>;

impl<'th, T> TrieHarderSet<'th, T>
where
    T: OneCounter
        + UnsignedInt
        + Shl<T, Output = T>
        + AddAssign
        + BitAnd<T, Output = T>
        + Eq
        + BitOrAssign
        + Sub<T, Output = T>,
{
    pub fn from_strs(input: &[&'th [u8]]) -> Self {
        Self::from_strs_and_values(input, &vec![(); input.len()])
    }

    pub fn contains(&self, key: &[u8]) -> bool {
        self.get(key).is_some()
    }
}

#[derive(Debug)]
pub struct TrieHarderMap<'th, T, V> {
    lookup_table: LookupTable<T>,
    nodes: Vec<TrieNode<'th, T, V>>,
}

#[derive(Debug)]
pub struct LookupTable<T>([Option<T>; 256]);

pub trait UnsignedInt: Copy {
    fn zero() -> Self;
    fn one() -> Self;
}

macro_rules! impl_unsigned_int {
    ($($u: ty), *) => {
        $(
            impl UnsignedInt for $u {
                fn zero() -> Self {
                    0
                }
                fn one() -> Self {
                    1
                }
            }
        )*
    };
}

impl_unsigned_int!(u8, u16, u32, u64, u128);

impl<T: UnsignedInt> Index<u8> for LookupTable<T> {
    type Output = Option<T>;

    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}

pub trait OneCounter {
    fn ones_count(&self) -> u32;
}

macro_rules! impl_one_counter_int {
    ($($u: ty),*) => {
        $(impl OneCounter for $u {
            fn ones_count(&self) -> u32 {
                self.count_ones()
            }
        })*
    };
}

impl_one_counter_int!(u8, u16, u32, u64, u128);

#[derive(Debug)]
pub enum TrieNode<'th, T, V> {
    Branch(BranchNode<'th, T>),
    Leaf(LeafNode<'th, T, V>),
}

#[derive(Debug)]
pub struct LeafNode<'th, T, V> {
    index: usize,
    string: &'th [u8],
    mask: T,
    children: Vec<usize>,
    value: V,
}

#[derive(Debug)]
pub struct BranchNode<'th, T> {
    index: usize,
    string: &'th [u8],
    mask: T,
    children: Vec<usize>,
}

impl<'th, T: UnsignedInt, V> TrieNode<'th, T, V> {
    pub fn new_branch(data: &'th [u8], index: usize) -> Self {
        Self::Branch(BranchNode {
            index,
            string: data,
            mask: T::zero(),
            children: vec![],
        })
    }

    pub fn new_leaf(data: &'th [u8], index: usize, value: V) -> Self {
        Self::Leaf(LeafNode {
            index,
            string: data,
            mask: T::zero(),
            children: vec![],
            value,
        })
    }

    pub fn index_mut(&mut self) -> &mut usize {
        match self {
            TrieNode::Branch(n) => &mut n.index,
            TrieNode::Leaf(n) => &mut n.index,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            TrieNode::Branch(n) => n.index,
            TrieNode::Leaf(n) => n.index,
        }
    }

    pub fn mask_mut(&mut self) -> &mut T {
        match self {
            TrieNode::Branch(n) => &mut n.mask,
            TrieNode::Leaf(n) => &mut n.mask,
        }
    }

    pub fn mask(&self) -> T {
        match self {
            TrieNode::Branch(n) => n.mask,
            TrieNode::Leaf(n) => n.mask,
        }
    }

    pub fn children_mut(&mut self) -> &mut Vec<usize> {
        match self {
            TrieNode::Branch(n) => &mut n.children,
            TrieNode::Leaf(n) => &mut n.children,
        }
    }

    pub fn children(&self) -> &[usize] {
        match self {
            TrieNode::Branch(n) => &n.children,
            TrieNode::Leaf(n) => &n.children,
        }
    }

    pub fn add_child(&mut self, child: &TrieNode<'_, T, V>, lookup: &LookupTable<T>)
    where
        T: BitOrAssign<T>,
    {
        let (inner_string, inner_index) = match child {
            TrieNode::Branch(n) => (n.string, n.index),
            TrieNode::Leaf(n) => (n.string, n.index),
        };
        match self {
            TrieNode::Branch(n) => {
                let c = inner_string[inner_string.len() - 1];
                n.mask |= lookup[c].expect("no character find in lookup talbe");
                n.children.push(inner_index);
            }
            TrieNode::Leaf(n) => {
                let c = inner_string[inner_string.len() - 1];
                n.mask |= lookup[c].expect("no character find in lookup table");
                n.children.push(inner_index);
            }
        }
    }
}

impl<'th, T, V> TrieHarderMap<'th, T, V>
where
    V: Clone,
    T: OneCounter
        + UnsignedInt
        + Shl<T, Output = T>
        + AddAssign
        + BitAnd<T, Output = T>
        + Eq
        + BitOrAssign
        + Sub<T, Output = T>,
{
    pub fn from_strs_and_values(input: &[&'th [u8]], values: &[V]) -> Self {
        let mut i = 0;
        let mut mask_index = T::zero();
        let mut lookup_table: LookupTable<T> = LookupTable([None; 256]);
        loop {
            let mut is_done = true;
            for &data in input {
                if i >= data.len() {
                    continue;
                }
                is_done = false;
                let c = data[i];
                if lookup_table.0[c as usize].is_some() {
                    continue;
                }
                let mask = T::one() << mask_index;
                lookup_table.0[c as usize] = Some(mask);
                mask_index += T::one();
            }
            if is_done {
                break;
            }
            i += 1;
        }
        let root = TrieNode::new_branch(&[], 0);
        let mut node_index = 1;
        let mut last_node_index = 0;
        let mut nodes = vec![root];
        for (&data, value) in input.iter().zip(values) {
            for (i, &c) in data.iter().enumerate() {
                let mut node = if i < data.len() - 1 {
                    TrieNode::new_branch(&data[..i + 1], node_index)
                } else {
                    TrieNode::new_leaf(data, node_index, value.clone())
                };
                if lookup_table[c].unwrap() & nodes[last_node_index].mask() == T::zero() {
                    nodes[last_node_index].add_child(&node, &lookup_table);
                    nodes.push(node);
                    last_node_index = node_index;
                    node_index += 1;
                } else {
                    let child_index = ((lookup_table[c].unwrap() - T::one())
                        & nodes[last_node_index].mask())
                    .ones_count();

                    last_node_index = nodes[last_node_index].children()[child_index as usize];
                    if i == data.len() - 1 {
                        assert!(matches!(node, TrieNode::Leaf(_)));
                        *node.index_mut() = nodes[last_node_index].index();
                        *node.mask_mut() = nodes[last_node_index].mask();
                        node.children_mut()
                            .extend(nodes[last_node_index].children());
                        let _ = std::mem::replace(&mut nodes[last_node_index], node);
                    }
                }
            }
            last_node_index = 0;
        }
        Self {
            lookup_table,
            nodes,
        }
    }

    pub fn get(&self, input: &[u8]) -> Option<&V> {
        match self.find_node(input) {
            Some(TrieNode::Leaf(n)) => Some(&n.value),
            _ => None,
        }
    }

    pub fn has_prefix(&self, input: &[u8]) -> bool {
        matches!(self.find_node(input), Some(TrieNode::Branch(_)))
    }

    fn find_node(&self, input: &[u8]) -> Option<&TrieNode<'_, T, V>>
    where
        T: UnsignedInt + BitAnd<T, Output = T> + Eq + Sub<T, Output = T> + OneCounter,
    {
        let mut node = &self.nodes[0];
        for &c in input {
            let c_mask = self.lookup_table[c]?;
            if (c_mask & node.mask()) == T::zero() {
                return None;
            }
            let child_index = ((c_mask - T::one()) & node.mask()).ones_count();
            let next_node_index = node.children()[child_index as usize];
            node = &self.nodes[next_node_index];
        }
        Some(node)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_lookup_table() {
        let words: [&[u8]; 5] = [b"and", b"ant", b"dad", b"do", b"dot"];
        let th: TrieHarderSet<'_, u8> = TrieHarderSet::from_strs(&words);
        assert_eq!(th.lookup_table.0[b'a' as usize], Some(1));
        assert_eq!(th.lookup_table.0[b'd' as usize], Some(2));
        assert_eq!(th.lookup_table.0[b'n' as usize], Some(4));
        assert_eq!(th.lookup_table.0[b'o' as usize], Some(8));
        assert_eq!(th.lookup_table.0[b't' as usize], Some(16));
        assert!(th.lookup_table.0[b'-' as usize].is_none());
    }

    #[test]
    fn test_create_trie_harder() {
        let words: [&[u8]; 5] = [b"and", b"ant", b"dad", b"do", b"dot"];
        let th: TrieHarderMap<'_, u8, ()> = TrieHarderSet::from_strs(&words);
        assert_eq!(th.nodes.len(), 10);
        let root = match &th.nodes[0] {
            TrieNode::Branch(n) => n,
            TrieNode::Leaf(_) => panic!("should not be leaf"),
        };
        assert_eq!(root.string, b"");
        assert_eq!(root.mask, 0b11);
        assert_eq!(root.children, [1, 5]);
        let node_a = match &th.nodes[1] {
            TrieNode::Branch(n) => n,
            TrieNode::Leaf(_) => panic!("should not be leaf"),
        };
        assert_eq!(node_a.string, b"a");
        assert_eq!(node_a.mask, 0b100);
        assert_eq!(node_a.children, [2]);
        let node_dad = match &th.nodes[7] {
            TrieNode::Branch(_) => panic!("should not be branch"),
            TrieNode::Leaf(n) => n,
        };
        assert_eq!(node_dad.string, b"dad");
        assert_eq!(node_dad.mask, 0);
        assert_eq!(node_dad.children, []);
    }

    #[test]
    fn test_trie_harder_get() {
        let words: [&[u8]; 5] = [b"and", b"ant", b"dad", b"dot", b"do"];
        let th: TrieHarderSet<'_, u8> = TrieHarderSet::from_strs(&words);
        assert!(th.contains(b"and"));
        assert!(th.contains(b"ant"));
        assert!(th.contains(b"dad"));
        assert!(th.contains(b"do"));
        assert!(th.contains(b"dot"));
        assert!(!th.contains(b"a"));
        assert!(!th.contains(b"an"));
        assert!(!th.contains(b"d"));
        assert!(!th.contains(b"da"));
    }

    #[test]
    fn test_trie_harder_map() {
        let paths: [&[u8]; 3] = [b"/static/js/", b"/web/index", b"/images/"];
        #[derive(Debug, Clone)]
        struct Config {
            upstreams: Vec<String>,
            path_map: String,
        }
        let configs: [Config; 3] = [
            Config {
                upstreams: vec!["127.0.0.1:8080".into(), "192.168.0.254:80".into()],
                path_map: "/js".into(),
            },
            Config {
                upstreams: vec!["127.0.0.100:8088".into(), "192.168.0.253:443".into()],
                path_map: "/web".into(),
            },
            Config {
                upstreams: vec!["127.0.0.10:8089".into(), "192.168.0.25:8081".into()],
                path_map: "/".into(),
            },
        ];
        let th: TrieHarderMap<'_, u16, Config> =
            TrieHarderMap::from_strs_and_values(&paths, &configs);
        assert_eq!(th.get(b"/web/index").unwrap().path_map, "/web");
        assert_eq!(
            th.get(b"/images/").unwrap().upstreams,
            ["127.0.0.10:8089", "192.168.0.25:8081"]
        );
    }
}
