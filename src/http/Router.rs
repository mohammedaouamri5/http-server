// Cargo.toml: regex = "1"
use regex::Regex;
use std::collections::HashMap;

use crate::http::{Context::HTTPContext, Error::HTTPError};

type HandlerFunc = fn(&mut HTTPContext) -> Result<(), HTTPError>;
// this will have a list of routes and the function handler of each
#[derive(Debug)]
struct Router {
    middlewares: Vec<HandlerFunc>,
    // routes per method
    get: Vec<Route>,
    post: Vec<Route>,
    is_compiled: bool,
}

#[derive(Debug, Clone)]
struct Route {
    path: String,
    method: String,
    handler: HandlerFunc,
    middlewares: Vec<HandlerFunc>,
    // built by Router::compile()
    regex: Option<Regex>,
}

impl Router {
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
        middlewares: &[HandlerFunc],
    ) {
        self.get.push(Route::new(path, "GET", handler, middlewares));
    }

    pub fn POST(
        &mut self,
        path: impl Into<String>,
        handler: HandlerFunc,
        middlewares: &[HandlerFunc],
    ) {
        self.post
            .push(Route::new(path, "POST", handler, middlewares));
    }

    // turns every route path into a regex so lookups are just a matching loop
    pub fn compile(&mut self) {
        for route in self.get.iter_mut().chain(self.post.iter_mut()) {
            route.regex = Some(Route::path_to_regex(&route.path));
        }
        self.is_compiled = true;
    }

    // finds the matching route and extracts the path params
    fn get_route(&self, method: &str, path: &str) -> Option<(&Route, HashMap<String, String>)> {
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
                return Some((route, params));
            }
        }
        None
    }

    // look up the route, run global middlewares, then the route itself
    pub fn route(&self, method: &str, path: &str) -> Option<HashMap<String, String>> {
        let (route, params) = self.get_route(method, path)?;

        for middleware in self.middlewares.iter() {
            middleware();
        }
        route.run();
        Some(params)
    }
    pub fn add_middleware(&mut self, middleware: HandlerFunc) {
        self.middlewares.push(middleware);
    }
}

impl Route {
    fn new(
        path: impl Into<String>,
        method: impl Into<String>,
        handler: HandlerFunc,
        middlewares: &[HandlerFunc],
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

    fn run(&self) {
        for middleware in self.middlewares.iter() {
            middleware();
        }
        (self.handler)();
    }
}

/*
 *  fn main() {
 *      let mut router = Router::new();
 *      router.middlewares.push(log);
 *      router.GET("/", index, &[]);
 *      router.GET("/users/:id", user, &[]);
 *      router.GET("/users/:id/posts/:post_id", user, &[]);
 *      router.compile();
 *
 *      match router.route("GET", "/users/42/posts/7") {
 *          Some(params) => println!("{params:?}"), // {"id": "42", "post_id": "7"}
 *          None => println!("404"),
 *      }
 *  }
*/
