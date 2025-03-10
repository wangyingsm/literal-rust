use super::path::Query;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Boundary {
    RawText {
        name: Option<String>,
        filename: String,
        content: String,
    },
    RawBinary {
        name: Option<String>,
        filename: String,
        content: Vec<u8>,
    },
    FormData(Query),
}

impl Boundary {
    pub fn new_raw_text(name: Option<String>, filename: String, text: String) -> Self {
        Self::RawText {
            name,
            filename,
            content: text,
        }
    }

    pub fn new_raw_binary(name: Option<String>, filename: String, binary: Vec<u8>) -> Self {
        Self::RawBinary {
            name,
            filename,
            content: binary,
        }
    }

    // pub fn new_form_data(data: &str) -> Self {
    //     let mut query = vec![];
    //     let mut query_map = HashMap::new();
    //     for line in data.lines() {
    //         let mut nv = line.split('=');
    //         if let Some(name) = nv.next() {
    //             if let Some(value) = nv.next() {
    //                 query_map
    //                     .entry(name)
    //                     .or_insert(vec![])
    //                     .push(value.to_string());
    //             }
    //         }
    //     }
    //     for (name, mut value) in query_map.into_iter() {
    //         if value.len() == 1 {
    //             query.push(Query::Single {
    //                 name: name.to_string(),
    //                 value: std::mem::take(&mut value[0]),
    //             });
    //         } else {
    //             query.push(Query::Multi {
    //                 name: name.to_string(),
    //                 value,
    //             });
    //         }
    //     }
    //     Self::FormData(query)
    // }
}
