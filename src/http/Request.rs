use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

use crate::http::Error::HTTPError;
use crate::http_err;

#[derive(Debug)]
pub struct HTTPRequest {
    pub method: String,
    pub url: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HTTPRequest {
    pub fn from_stream(stream: &TcpStream) -> Result<Self, HTTPError> {
        let mut reader = BufReader::new(stream);

        // TODO : MAX REQUEST

        // start line
        // TODO : MAX LINE
        let mut request_line = String::new();
        match reader.read_line(&mut request_line) {
            ok => (),
            Err(e) => return Err(HTTPError::Io(e)),
        }

        let mut parts = request_line.split_whitespace();
        let method = parts
            .next()
            .ok_or_else(|| http_err!(BadRequest, "cant get method from request"))?;
        let mut url = parts
            .next()
            .ok_or_else(|| http_err!(BadRequest, "cant get url from request"))?;
        let version = parts
            .next()
            .ok_or_else(|| http_err!(BadRequest, "cant get version from request"))?;

        // headers
        let mut headers = String::new();
        let mut headers_map: Vec<(String, String)> = Vec::new();
        // TODO : MAX LINE
        // TODO : MAX HEADERS
        loop {
            let mut line = String::new();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 || line == "\r\n" {
                break;
            }
            headers.push_str(&line);
            let (key, value) = line
                .split_once(':')
                .ok_or_else(|| http_err!(BadRequest, "bad header"))?;
            headers_map.push((key.trim().to_string(), value.trim().to_string()));
        }

        // body
        let content_length = match headers_map
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("Content-Length"))
        {
            Some((_, v)) => v
                .parse::<usize>()
                .map_err(|_| http_err!(BadRequest, "bad content length"))?,
            None => 0,
        };

        let mut body = vec![0u8; content_length];
        reader.read_exact(&mut body)?;

        let headers = headers_map.into_iter().collect();

        Ok(Self {
            method: method.to_string(),
            url: url.to_string(),
            version: version.to_string(),
            headers,
            body,
        })
    }
}
