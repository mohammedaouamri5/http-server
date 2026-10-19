use std::{collections::HashMap, io::Write, net::TcpStream, string};

use crate::{
    constants::response,
    http::{
        Error::HTTPError::{self, InternalServerError},
        Request::HTTPRequest,
        Response::HTTPResponse,
    },
    http_err,
};
#[derive(Debug)]
pub struct HTTPContext {
    stream: TcpStream,
    request: HTTPRequest,
    response: HTTPResponse,
    Data: HashMap<String, String>,
}

impl HTTPContext {
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
            Data: data,
        })
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
        if self.request.url.is_empty() {
            return Err(http_err!(InternalServerError, "can get the URL"));
        }
        return Ok(&self.request.method);
    }
}
