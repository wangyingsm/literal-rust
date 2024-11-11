use std::ops::Index;

#[derive(Debug)]
pub struct TrieHarder<'th> {
    lookup_table: LookupTable,
    nodes: Vec<TrieNode<'th>>,
}

#[derive(Debug)]
pub struct LookupTable([Option<u32>; 256]);

impl Index<u8> for LookupTable {
    type Output = Option<u32>;

    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}

#[derive(Debug)]
pub enum TrieNode<'th> {
    Branch(InnerNode<'th>),
    Leaf(InnerNode<'th>),
}

#[derive(Debug)]
pub struct InnerNode<'th> {
    index: usize,
    string: &'th [u8],
    mask: u32,
    children: Vec<usize>,
}

impl<'th> TrieNode<'th> {
    pub fn new_branch(data: &'th [u8], index: usize) -> Self {
        Self::Branch(InnerNode {
            index,
            string: data,
            mask: 0,
            children: vec![],
        })
    }

    pub fn new_leaf(data: &'th [u8], index: usize) -> Self {
        Self::Leaf(InnerNode {
            index,
            string: data,
            mask: 0,
            children: vec![],
        })
    }

    pub fn mask(&self) -> u32 {
        match self {
            TrieNode::Branch(n) => n.mask,
            TrieNode::Leaf(n) => n.mask,
        }
    }

    pub fn children(&self) -> &[usize] {
        match self {
            TrieNode::Branch(n) => &n.children,
            TrieNode::Leaf(n) => &n.children,
        }
    }

    pub fn add_child(&mut self, child: &TrieNode, lookup: &LookupTable) {
        let inner = match child {
            TrieNode::Branch(n) => n,
            TrieNode::Leaf(n) => n,
        };
        match self {
            TrieNode::Branch(n) => {
                let c = inner.string[inner.string.len() - 1];
                n.mask |= lookup[c].expect("no character find in lookup talbe");
                n.children.push(inner.index);
            }
            TrieNode::Leaf(n) => {
                let c = inner.string[inner.string.len() - 1];
                n.mask |= lookup[c].expect("no character find in lookup table");
                n.children.push(inner.index);
            }
        }
    }
}

impl<'th> TrieHarder<'th> {
    pub fn from_strs(input: &[&'th [u8]]) -> Self {
        let mut i = 0;
        let mut mask_index = 0;
        let mut lookup_table = LookupTable([None; 256]);
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
                let mask = 1 << mask_index;
                lookup_table.0[c as usize] = Some(mask);
                mask_index += 1;
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
        for &data in input {
            for (i, &c) in data.iter().enumerate() {
                let node = if i < data.len() - 1 {
                    TrieNode::new_branch(&data[..i + 1], node_index)
                } else {
                    TrieNode::new_leaf(data, node_index)
                };
                if lookup_table[c].unwrap() & nodes[last_node_index].mask() == 0 {
                    nodes[last_node_index].add_child(&node, &lookup_table);
                    nodes.push(node);
                    last_node_index = node_index;
                    node_index += 1;
                } else {
                    let child_index = ((lookup_table[c].unwrap() - 1)
                        & nodes[last_node_index].mask())
                    .count_ones();
                    last_node_index = nodes[last_node_index].children()[child_index as usize];
                }
            }
            last_node_index = 0;
        }
        Self {
            lookup_table,
            nodes,
        }
    }

    pub fn get(&self, input: &[u8]) -> Option<usize> {
        todo!();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_lookup_table() {
        let words: [&[u8]; 5] = [b"and", b"ant", b"dad", b"do", b"dot"];
        let th = TrieHarder::from_strs(&words);
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
        let th = TrieHarder::from_strs(&words);
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
}
