use std::collections::HashMap;

use super::path::Query;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Boundary {
    RawText(String),
    RawBinary(Vec<u8>),
    FormData(Vec<Query>),
}

impl Boundary {
    pub fn new_raw_text(text: String) -> Self {
        Self::RawText(text)
    }

    pub fn new_raw_binary(binary: Vec<u8>) -> Self {
        Self::RawBinary(binary)
    }

    pub fn new_form_data(data: &str) -> Self {
        let mut query = vec![];
        let mut query_map = HashMap::new();
        for line in data.lines() {
            let mut nv = line.split('=');
            if let Some(name) = nv.next() {
                if let Some(value) = nv.next() {
                    query_map
                        .entry(name)
                        .or_insert(vec![])
                        .push(value.to_string());
                }
            }
        }
        for (name, mut value) in query_map.into_iter() {
            if value.len() == 1 {
                query.push(Query::Single {
                    name: name.to_string(),
                    value: std::mem::take(&mut value[0]),
                });
            } else {
                query.push(Query::Multi {
                    name: name.to_string(),
                    value,
                });
            }
        }
        Self::FormData(query)
    }
}
