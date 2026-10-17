use std::{collections::HashMap, io::Write, net::TcpStream};

use crate::{
    constants::response,
    http::{Error::HTTPError, Request::HTTPRequest, Response::HTTPResponse},
};

#[derive(Debug)]
pub struct HTTPContext<'a> {
    stream: &'a mut TcpStream,
    request: HTTPRequest,
    response: HTTPResponse,
    Data: HashMap<String, String>,
}

impl<'a> HTTPContext<'a> {
    pub fn new(stream: &'a mut TcpStream) -> Result<Self, HTTPError> {
        let request = match HTTPRequest::from_stream(stream) {
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
}


