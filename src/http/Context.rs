use std::{collections::HashMap, io::Write, net::TcpStream};

use crate::{
    constants::response::{self, OK},
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

// 'a is the router's lifetime: the context borrows the middleware list from it
#[derive(Debug)]
pub struct HTTPContext<'a> {
    stream: TcpStream,
    request: HTTPRequest,
    response: HTTPResponse,
    middlewares: Option<&'a Vec<Middleware>>,
    i_middlewares: Option<usize>,
    handler: Option<HandlerFunc>,
    params: Option<HashMap<String, String>>,
    written: bool,
}

impl<'a> HTTPContext<'a> {
    pub fn new(mut stream: TcpStream) -> Result<Self, HTTPError> {
        let request = match HTTPRequest::from_stream(&stream) {
            Ok(request) => request,
            Err(err) => {
                Self::_suspend(&err, &mut stream);
                return Err(err);
            }
        };
        let mut response = HTTPResponse::new();
        response.set_status(OK);

        Ok(Self {
            stream,
            request,
            response,
            middlewares: None,
            i_middlewares: None,
            handler: None,
            params: None,
            written: false,
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
            Some((handler, params, middlewares)) => {
                self.handler = Some(handler);
                self.middlewares = Some(middlewares);
                self.params = Some(params);
            }
            None => {
                return;
            }
        }
    }

    pub fn run(&mut self) -> Result<(), HTTPError> {
        let result = self.dispatch();
        if let Err(err) = &result {
            if !self.written && err.status_line().is_some() {
                Self::_suspend(err, &mut self.stream);
                self.written = true;
            }
        }
        result
    }

    // runs the middlewares then the handler, and writes the response once
    fn dispatch(&mut self) -> Result<(), HTTPError> {
        if self.handler.is_none() {
            return Err(http_err!(
                NotFound,
                "no route for {} {}",
                self.request.method,
                self.request.url
            ));
        }
        self.i_middlewares = Some(0);
        self.Next()?;
        if !self.written {
            let body = self.response.buffer()?;
            self.stream.write_all(&body)?;
            self.written = true;
        }
        Ok(())
    }

    pub fn Next(&mut self) -> Result<(), HTTPError> {
        if self.i_middlewares.is_some() {
            let i = self.i_middlewares.unwrap();
            if i < self.middlewares.unwrap().len() {
                self.i_middlewares = Some(i + 1);
                let middleware = self.middlewares.unwrap()[i];
                match middleware(self) {
                    Flow::Next => {}
                    Flow::Break => self.i_middlewares = None,
                    Flow::Error(err) => return Err(err),
                }
                return Ok(());
            }
            self.i_middlewares = None;
        }
        // take() makes sure the handler runs only once, even if Next is called again
        if let Some(handler) = self.handler.take() {
            handler(self)?;
        }
        Ok(())
    }

    pub fn Break(&mut self) {
        let body = self.response.buffer().unwrap();
        self.stream.write_all(&body).unwrap();
        self.written = true;
    }

    pub fn Error(&mut self, err: HTTPError) {
        Self::_suspend(&err, &mut self.stream);
        self.written = true;
    }

    fn _suspend(__error: &HTTPError, stream: &mut TcpStream) {
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

    pub fn get_body(&self) -> &Vec<u8> {
        &self.request.body
    }

    pub fn get_param(&self, name: &str) -> Option<&String> {
        self.params.as_ref()?.get(name)
    }

    pub fn text(&mut self, body: &str) {
        self.response.text(body);
    }

    pub fn json<T: serde::Serialize>(&mut self, value: &T) -> Result<(), HTTPError> {
        self.response.json(value)?;
        Ok(())
    }
    pub fn send_file(&mut self, path: &str) -> Result<(), HTTPError> {
        self.response.set_file(path)?;
        Ok(())
    }

    pub fn set_status(&mut self, status: &str) {
        self.response.set_status(status);
    }

    pub fn suspend(&mut self) {
        let mut response = HTTPResponse::new();
        response.set_status(response::INTERNAL_SERVER_ERROR);
        let body = response.buffer().unwrap();
        self.stream.write_all(&body).unwrap();
    }
}
