use std::{collections::HashMap, io::Write, net::TcpStream};

use crate::{
    constants::response,
    http::{
        Error::{
            Flow,
            HTTPError::{self, InternalServerError},
        },
        Request::HTTPRequest,
        Response::HTTPResponse,
        Router::{HTTPRouter, HandlerFunc, Middleware},
    },
    http_err,
};

// 'a is the router's lifetime: the context borrows the middleware lists from it
#[derive(Debug)]
pub struct HTTPContext<'a> {
    stream: TcpStream,
    request: HTTPRequest,
    response: HTTPResponse,
    router_middlewares: Option<&'a Vec<Middleware>>,
    i_router_middlewares:Option<i8>
    route_middlewares: Option<&'a Vec<Middleware>>,
    i_route_middlewares: Option<i8>,
    handler: Option<HandlerFunc>,
    params: Option<HashMap<String, String>>,
    Data: HashMap<String, String>,
}

impl<'a> HTTPContext<'a> {
    pub fn new(mut stream: TcpStream) -> Result<Self, HTTPError> {
        let request = match HTTPRequest::from_stream(&stream) {
            Ok(request) => request,
            Err(err) => {
                Self::_suspend(&err, stream);
                return Err(err);
            }
        };
        let data = HashMap::new();
        let response = HTTPResponse::new();

        Ok(Self {
            stream,
            request,
            response,
            route_middlewares: None,
            i_route_middlewares: None,
            router_middlewares: None,
            i_router_middlewares: None,
            handler: None,
            params: None,
            Data: data,
        })
    }

    pub fn route(&mut self, router: &'a HTTPRouter) {
        assert_eq!(
            self.request.url.is_empty(),
            false,
            "cant get the route witout the url"
        );
        assert_eq!(
            self.request.method.is_empty(),
            false,
            "cant get the route witout the method"
        );

        match (router.get_route(self.request.method.as_str(), self.request.url.as_str())) {
            Some((handler, params, router_middlewares, route_middlewares)) => {
                self.handler = Some(handler);
                self.router_middlewares = Some(router_middlewares);
                self.route_middlewares = Some(route_middlewares);
                self.params = Some(params);
            }
            None => {
                return;
            }
        }
    }

    pub fn run(&mut self) -> Result<(), HTTPError> {
        let Some(handler) = self.handler else {
            return Err(http_err!(InternalServerError, "no route matched"));
        };

    }

    fn _suspend(__error: &HTTPError, mut stream: TcpStream) {
        let mut response = HTTPResponse::new();
        response.set_status(
            __error
                .status_line()
                .unwrap_or(response::INTERNAL_SERVER_ERROR),
        );
        let body = response.buffer().unwrap();
        stream.write_all(&body).unwrap();
    }

    pub fn get_url(&self) -> Result<&String, HTTPError> {
        if self.request.url.is_empty() {
            return Err(http_err!(InternalServerError, "can get the URL"));
        }
        return Ok(&self.request.url);
    }

    pub fn get_method(&self) -> Result<&String, HTTPError> {
        if self.request.method.is_empty() {
            return Err(http_err!(InternalServerError, "can get the method"));
        }
        return Ok(&self.request.method);
    }

    pub fn suspend(&mut self) {
        let mut response = HTTPResponse::new();
        response.set_status(response::INTERNAL_SERVER_ERROR);
        let body = response.buffer().unwrap();
        self.stream.write_all(&body).unwrap();
    }
}
