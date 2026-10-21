use regex::Regex;
use std::collections::HashMap;

use crate::http::{
    Context::HTTPContext,
    Error::{Flow, HTTPError},
};

pub type HandlerFunc = fn(&mut HTTPContext<'_>) -> Result<(), HTTPError>;
pub type Middleware = fn(&mut HTTPContext<'_>);
// this will have a list of routes and the function handler of each
#[derive(Debug)]
pub struct HTTPRouter {
    middlewares: Vec<Middleware>,
    // routes per method
    get: Vec<HTTPRoute>,
    post: Vec<HTTPRoute>,
    is_compiled: bool,
}

#[derive(Debug)]
pub struct HTTPRoute {
    path: String,
    method: String,
    handler: HandlerFunc,
    middlewares: Vec<Middleware>,
    // built by Router::compile()
    regex: Option<Regex>,
}

impl HTTPRouter {
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
            get: Vec::new(),
            post: Vec::new(),
            is_compiled: false,
        }
    }

    pub fn GET(
        &mut self,
        path: impl Into<String>,
        handler: HandlerFunc,
        middlewares: &[Middleware],
    ) {
        self.get
            .push(HTTPRoute::new(path, "GET", handler, middlewares));
    }

    pub fn POST(
        &mut self,
        path: impl Into<String>,
        handler: HandlerFunc,
        middlewares: &[Middleware],
    ) {
        self.post
            .push(HTTPRoute::new(path, "POST", handler, middlewares));
    }

    // turns every route path into a regex and merges the global middlewares
    // into each route, so every route has one list to run
    pub fn compile(&mut self) {
        assert!(!self.is_compiled, "compile() called twice");

        for route in self.get.iter_mut().chain(self.post.iter_mut()) {
            route.regex = Some(HTTPRoute::path_to_regex(&route.path));
            // global middlewares first, then the route's own
            route.middlewares = self
                .middlewares
                .iter()
                .chain(route.middlewares.iter())
                .copied()
                .collect();
        }
        self.is_compiled = true;
    }

    pub fn get_route(
        &self,
        method: &str,
        path: &str,
    ) -> Option<(HandlerFunc, HashMap<String, String>, &Vec<Middleware>)> {
        assert!(self.is_compiled, "call compile() first");

        let routes = match method {
            "GET" => &self.get,
            "POST" => &self.post,
            _ => return None,
        };

        for route in routes {
            let Some(re) = &route.regex else { continue };
            if let Some(caps) = re.captures(path) {
                let params = re
                    .capture_names()
                    .flatten() // skips unnamed groups
                    .filter_map(|name| {
                        caps.name(name)
                            .map(|m| (name.to_string(), m.as_str().to_string()))
                    })
                    .collect();

                return Some((route.handler, params, &route.middlewares));
            }
        }
        None
    }

    pub fn add_middlewares(&mut self, middlewares: &[Middleware]) {
        self.middlewares.extend_from_slice(middlewares);
    }
}

impl HTTPRoute {
    fn new(
        path: impl Into<String>,
        method: impl Into<String>,
        handler: HandlerFunc,
        middlewares: &[Middleware],
    ) -> Self {
        Self {
            path: path.into(),
            method: method.into(),
            handler,
            middlewares: middlewares.to_vec(),
            regex: None,
        }
    }

    // "/users/:id/posts/:post_id" -> ^/users/(?P<id>[^/]+)/posts/(?P<post_id>[^/]+)/?$
    fn path_to_regex(path: &str) -> Regex {
        let pattern = path
            .trim_end_matches('/')
            .split('/')
            .map(|seg| match seg.strip_prefix(':') {
                Some(name) => format!("(?P<{name}>[^/]+)"),
                None => regex::escape(seg),
            })
            .collect::<Vec<_>>()
            .join("/");
        Regex::new(&format!("^{pattern}/?$")).expect("invalid route path")
    }
}

/*
 *  fn main() {
 *      let mut router = HTTPRouter::new();
 *      router.GET("/", index, &[]);
 *      router.GET("/users/:id", user, &[]);
 *      router.GET("/users/:id/posts/:post_id", user, &[]);
 *      router.compile();
 *
 *      // leak it once so every thread can hold a &'static HTTPRouter
 *      let router: &'static HTTPRouter = Box::leak(Box::new(router));
 *  }
*/
