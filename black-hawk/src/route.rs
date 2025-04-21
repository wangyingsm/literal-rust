use std::collections::HashMap;

use radix_trie::Trie;

use crate::{
    handler::Handler,
    request::{HttpMethod, HttpRequest},
};

pub trait Router {
    fn route(&self, request: &mut HttpRequest) -> Option<&dyn Handler>;
}

pub struct StaticRouter {
    get: Trie<&'static str, Box<dyn Handler>>,
    post: Trie<&'static str, Box<dyn Handler>>,
    put: Trie<&'static str, Box<dyn Handler>>,
    options: Trie<&'static str, Box<dyn Handler>>,
    delete: Trie<&'static str, Box<dyn Handler>>,
    patch: Trie<&'static str, Box<dyn Handler>>,
    head: Trie<&'static str, Box<dyn Handler>>,
    connect: Trie<&'static str, Box<dyn Handler>>,
    trace: Trie<&'static str, Box<dyn Handler>>,
}

impl Router for StaticRouter {
    fn route(&self, request: &mut HttpRequest) -> Option<&dyn Handler> {
        let path = request.header().path.abs_path();
        match request.header().method {
            crate::request::HttpMethod::Get => route_with_prefix(&self.get, path),
            crate::request::HttpMethod::Post => route_with_prefix(&self.post, path),
            crate::request::HttpMethod::Put => route_with_prefix(&self.put, path),
            crate::request::HttpMethod::Options => route_with_prefix(&self.options, path),
            crate::request::HttpMethod::Delete => route_with_prefix(&self.delete, path),
            crate::request::HttpMethod::Patch => route_with_prefix(&self.patch, path),
            crate::request::HttpMethod::Connect => route_with_prefix(&self.connect, path),
            crate::request::HttpMethod::Head => route_with_prefix(&self.head, path),
            crate::request::HttpMethod::Trace => route_with_prefix(&self.trace, path),
        }
    }
}

fn route_with_prefix<'t>(
    trie: &'t Trie<&'static str, Box<dyn Handler>>,
    path: &str,
) -> Option<&'t dyn Handler> {
    let mut path_parts: Vec<_> = path.split('/').collect();
    loop {
        // TODO: do we really need these 2 String allocation?
        let path = path_parts.join("/");
        if let Some(h) = trie.get(path.as_str()) {
            return Some(&**h);
        }
        path_parts.pop();
        path_parts.push("*");
        let path = path_parts.join("/");
        if let Some(h) = trie.get(path.as_str()) {
            return Some(&**h);
        }
        path_parts.pop();
        if path_parts.len() <= 1 {
            return trie.get("/").map(|h| &**h);
        }
    }
}

impl StaticRouter {
    pub fn new() -> Self {
        Self {
            get: Trie::new(),
            post: Trie::new(),
            put: Trie::new(),
            options: Trie::new(),
            delete: Trie::new(),
            patch: Trie::new(),
            head: Trie::new(),
            connect: Trie::new(),
            trace: Trie::new(),
        }
    }

    pub fn add_route(
        &mut self,
        method: &HttpMethod,
        path: &'static str,
        handler: Box<dyn Handler>,
    ) {
        match method {
            HttpMethod::Get => {
                self.get.insert(path, handler);
            }
            HttpMethod::Post => {
                self.post.insert(path, handler);
            }
            HttpMethod::Put => {
                self.post.insert(path, handler);
            }
            HttpMethod::Options => {
                self.options.insert(path, handler);
            }
            HttpMethod::Delete => {
                self.delete.insert(path, handler);
            }
            HttpMethod::Patch => {
                self.patch.insert(path, handler);
            }
            HttpMethod::Connect => {
                self.connect.insert(path, handler);
            }
            HttpMethod::Head => {
                self.head.insert(path, handler);
            }
            HttpMethod::Trace => {
                self.trace.insert(path, handler);
            }
        }
    }
}

impl Default for StaticRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct DynamicRouter {
    get: Trie<String, Box<dyn Handler>>,
    post: Trie<String, Box<dyn Handler>>,
    put: Trie<String, Box<dyn Handler>>,
    options: Trie<String, Box<dyn Handler>>,
    delete: Trie<String, Box<dyn Handler>>,
    patch: Trie<String, Box<dyn Handler>>,
    head: Trie<String, Box<dyn Handler>>,
    connect: Trie<String, Box<dyn Handler>>,
    trace: Trie<String, Box<dyn Handler>>,

    path_vars: HashMap<String, String>,
}

fn route_with_path_variable<'t>(
    trie: &'t Trie<String, Box<dyn Handler>>,
    path_vars: &HashMap<String, String>,
    request: &mut HttpRequest,
) -> Option<&'t dyn Handler> {
    let mut prefix = String::from("/");
    let path = request.header().path.abs_path().to_string();
    for p in path.split('/').skip(1) {
        prefix.push_str("{}");
        if let Some(name) = path_vars.get(&prefix) {
            request.path_vars.insert(name.to_string(), p.to_string());
        } else {
            prefix.pop();
            prefix.pop();
            prefix.push_str(p);
        }
        if let Some(h) = trie.get(&prefix) {
            return Some(&**h);
        }
    }
    None
}

impl DynamicRouter {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_route(&mut self, method: HttpMethod, path: &'static str, handler: Box<dyn Handler>) {
        let mut out_path = String::new();
        for p in path.split('/') {
            if p.starts_with('{') && p.ends_with('}') {
                if p.len() == 2 {
                    eprintln!("error path variable config with empty var name");
                    return;
                }
                out_path.push_str("{}");
                self.path_vars
                    .insert(out_path.clone(), p[1..p.len() - 1].to_string());
            } else {
                out_path.push_str(p);
            }
            out_path.push('/');
        }
        let out_path = out_path[..out_path.len() - 1].to_string();
        match method {
            HttpMethod::Get => {
                self.get.insert(out_path, handler);
            }
            HttpMethod::Post => {
                self.post.insert(out_path, handler);
            }
            HttpMethod::Put => {
                self.put.insert(out_path, handler);
            }
            HttpMethod::Options => {
                self.options.insert(out_path, handler);
            }
            HttpMethod::Delete => {
                self.delete.insert(out_path, handler);
            }
            HttpMethod::Patch => {
                self.patch.insert(out_path, handler);
            }
            HttpMethod::Connect => {
                self.connect.insert(out_path, handler);
            }
            HttpMethod::Head => {
                self.head.insert(out_path, handler);
            }
            HttpMethod::Trace => {
                self.trace.insert(out_path, handler);
            }
        }
    }
}

impl Router for DynamicRouter {
    fn route(&self, request: &mut HttpRequest) -> Option<&dyn Handler> {
        match request.header.method {
            HttpMethod::Get => route_with_path_variable(&self.get, &self.path_vars, request),
            HttpMethod::Post => route_with_path_variable(&self.post, &self.path_vars, request),
            HttpMethod::Put => route_with_path_variable(&self.put, &self.path_vars, request),
            HttpMethod::Options => {
                route_with_path_variable(&self.options, &self.path_vars, request)
            }
            HttpMethod::Delete => route_with_path_variable(&self.delete, &self.path_vars, request),
            HttpMethod::Patch => route_with_path_variable(&self.patch, &self.path_vars, request),
            HttpMethod::Connect => {
                route_with_path_variable(&self.connect, &self.path_vars, request)
            }
            HttpMethod::Head => route_with_path_variable(&self.head, &self.path_vars, request),
            HttpMethod::Trace => route_with_path_variable(&self.trace, &self.path_vars, request),
        }
    }
}
