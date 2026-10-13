use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

const MAX_LINE: u64 = 8 * 1024;
const MAX_HEADERS: usize = 100;
const MAX_BODY: usize = 1024 * 1024;

#[derive(Debug)]
pub struct HTTPRequest {
    pub method: String,
    pub url: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HTTPRequest {
    pub fn from_stream(stream: &TcpStream) -> Result<Self> {
        let mut reader = BufReader::new(stream);

        // start line
        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;

        let mut parts = request_line.split_whitespace();
        let method = parts.next().ok_or_else(|| anyhow!("missing method"))?;
        let url = parts.next().ok_or_else(|| anyhow!("missing url"))?;
        let version = parts.next().ok_or_else(|| anyhow!("missing version"))?;

        // headers
        let mut headers = String::new();
        let mut headers_map: Vec<(String, String)> = Vec::new();
        loop {
            let mut line = String::new();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 || line == "\r\n" {
                break;
            }
            headers.push_str(&line);
            let (key, value) = line
                .split_once(':')
                .ok_or_else(|| anyhow!("bad header: {}", line.trim()))?;
            headers_map.push((key.trim().to_string(), value.trim().to_string()));
        }

        // body
        let content_length = match headers_map
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("Content-Length"))
        {
            Some((_, v)) => v
                .parse::<usize>()
                .map_err(|_| anyhow!("bad content-length"))?,
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
