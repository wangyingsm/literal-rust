use std::collections::HashMap;

use crate::{handler::Handler, request::path::HttpPath};

pub trait Router {
    fn route(&self, path: &HttpPath) -> Option<&dyn Handler>;
}

pub struct StaticRouter {
    routes: HashMap<String, Box<dyn Handler>>,
}

impl Router for StaticRouter {
    fn route(&self, path: &HttpPath) -> Option<&dyn Handler> {
        self.routes.get(path.abs_path()).map(|h| &**h)
    }
}

impl StaticRouter {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, path: String, handler: Box<dyn Handler>) {
        self.routes.insert(path, handler);
    }
}

impl Default for StaticRouter {
    fn default() -> Self {
        Self::new()
    }
}
