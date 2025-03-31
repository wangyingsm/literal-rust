use std::collections::HashMap;

use crate::{
    handler::Handler,
    request::{HttpMethod, HttpRequest},
};

pub trait Router {
    fn route(&self, request: &HttpRequest) -> Option<&dyn Handler>;
}

pub struct StaticRouter {
    get: HashMap<String, Box<dyn Handler>>,
    post: HashMap<String, Box<dyn Handler>>,
    put: HashMap<String, Box<dyn Handler>>,
    options: HashMap<String, Box<dyn Handler>>,
    delete: HashMap<String, Box<dyn Handler>>,
    patch: HashMap<String, Box<dyn Handler>>,
    head: HashMap<String, Box<dyn Handler>>,
    connect: HashMap<String, Box<dyn Handler>>,
    trace: HashMap<String, Box<dyn Handler>>,
}

impl Router for StaticRouter {
    fn route(&self, request: &HttpRequest) -> Option<&dyn Handler> {
        match request.header().method {
            crate::request::HttpMethod::Get => {
                self.get.get(request.header().path.abs_path()).map(|h| &**h)
            }
            crate::request::HttpMethod::Post => self
                .post
                .get(request.header().path.abs_path())
                .map(|h| &**h),
            crate::request::HttpMethod::Put => {
                self.put.get(request.header().path.abs_path()).map(|h| &**h)
            }
            crate::request::HttpMethod::Options => self
                .options
                .get(request.header().path.abs_path())
                .map(|h| &**h),
            crate::request::HttpMethod::Delete => self
                .delete
                .get(request.header().path.abs_path())
                .map(|h| &**h),
            crate::request::HttpMethod::Patch => self
                .patch
                .get(request.header().path.abs_path())
                .map(|h| &**h),
            crate::request::HttpMethod::Connect => self
                .connect
                .get(request.header().path.abs_path())
                .map(|h| &**h),
            crate::request::HttpMethod::Head => self
                .head
                .get(request.header().path.abs_path())
                .map(|h| &**h),
            crate::request::HttpMethod::Trace => self
                .trace
                .get(request.header().path.abs_path())
                .map(|h| &**h),
        }
    }
}

impl StaticRouter {
    pub fn new() -> Self {
        Self {
            get: HashMap::new(),
            post: HashMap::new(),
            put: HashMap::new(),
            options: HashMap::new(),
            delete: HashMap::new(),
            patch: HashMap::new(),
            head: HashMap::new(),
            connect: HashMap::new(),
            trace: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, method: &HttpMethod, path: String, handler: Box<dyn Handler>) {
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
