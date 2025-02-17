use std::collections::HashMap;

use crate::error::RequestParseError;

#[derive(Debug, PartialEq, Eq)]
pub struct HttpPath {
    abs_path: String,
    query: Vec<Query>,
    anchor: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Query {
    Single { name: String, value: String },
    Multi { name: String, value: Vec<String> },
}

impl Query {
    pub fn to_single(self) -> Option<Self> {
        match self {
            Query::Single { name, value } => Some(Query::Single { name, value }),
            Query::Multi { name, mut value } => {
                if value.len() != 1 {
                    None
                } else {
                    Some(Query::Single {
                        name,
                        value: std::mem::take(&mut value[0]),
                    })
                }
            }
        }
    }
}

impl HttpPath {
    pub fn from_request(input: &[u8]) -> Result<Self, RequestParseError> {
        let input = String::from_utf8_lossy(input);
        let mut parts = input.split('?');

        let abs_path = if let Some(abs_path) = parts.next() {
            abs_path.to_string()
        } else {
            return Err(RequestParseError::EmptyRequestPath);
        };
        let mut query = HashMap::new();
        let mut query_vec = vec![];
        let anchor = if let Some(query_anchor) = parts.next() {
            let mut parts = query_anchor.split('#');
            if let Some(q) = parts.next() {
                let name_values = q.split('&');
                for nv_pair in name_values {
                    let mut nv = nv_pair.split('=');
                    if let Some(name) = nv.next() {
                        let value = nv.next().unwrap_or("");
                        query.entry(name).or_insert(vec![]).push(value.to_string());
                    } else {
                        return Err(RequestParseError::EmptyRequestPath);
                    }
                }
                for (k, mut v) in query.into_iter() {
                    if v.len() == 1 {
                        query_vec.push(Query::Single {
                            name: k.to_string(),
                            value: std::mem::take(&mut v[0]),
                        });
                    } else {
                        query_vec.push(Query::Multi {
                            name: k.to_string(),
                            value: v,
                        });
                    }
                }
            }
            parts.next().map(|anchor| anchor.to_string())
        } else {
            None
        };
        Ok(Self {
            abs_path,
            query: query_vec,
            anchor,
        })
    }

    pub fn abs_path(&self) -> &str {
        &self.abs_path
    }

    pub fn query(&self) -> &[Query] {
        &self.query
    }

    pub fn anchor(&self) -> Option<&str> {
        self.anchor.as_deref()
    }
}
