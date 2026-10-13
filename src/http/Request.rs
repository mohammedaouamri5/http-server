use anyhow::{Result, anyhow};
use core::fmt;
use log::{error, info};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::result;

#[derive(Debug)]
pub struct HTTPRequest {
    pub method: String,
    pub url: String,
    pub version: String,
}

impl HTTPRequest {
    pub fn from_stream(stream: &TcpStream) -> Result<Self> {
        // BufReader takes ownership of the &TcpStream, so no `mut` needed on yours
        let mut reader = BufReader::new(stream);

        // Only read the first line: "GET /index.html HTTP/1.1\r\n"
        let mut request_line = String::new();

        // start line
        reader.read_line(&mut request_line)?;

        let mut parts = request_line.split_whitespace();
        let method = parts.next().ok_or_else(|| anyhow!("missing method"))?;
        let url = parts.next().ok_or_else(|| anyhow!("missing url"))?;
        let version = parts.next().ok_or_else(|| anyhow!("missing version"))?;

        Ok(HTTPRequest {
            method: method.to_string(),
            url: url.to_string(),
            version: version.to_string(),
        })
    }
}
