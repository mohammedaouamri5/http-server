use anyhow::{Result, anyhow};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug)]
pub struct HTTPResponse {
    pub version: String,
    pub status: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HTTPResponse {
    pub fn new() -> Self {
        HTTPResponse {
            version: "HTTP/1.1".to_string(),
            status: "".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn set_header(&mut self, key: &str, value: &str) -> &mut Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Core setter: every other body helper goes through this.
    fn set_bytes(&mut self, content_type: &str, body: Vec<u8>) -> &mut Self {
        self.set_header("Content-Type", content_type);
        self.body = body;
        self
    }

    pub fn text(&mut self, body: &str) -> &mut Self {
        self.set_bytes("text/plain; charset=utf-8", body.as_bytes().to_vec())
    }

    pub fn set_status(&mut self, status: &str) -> &mut Self {
        self.status = status.to_string();
        self
    }
    pub fn json<T: Serialize>(&mut self, value: &T) -> Result<&mut Self> {
        let bytes = serde_json::to_vec(value)?;
        Ok(self.set_bytes("application/json", bytes))
    }

    pub fn set_file(&mut self, path: &str) -> Result<&mut Self> {
        // TODO : check if file exists
        // TODO : check redability

        let bytes = fs::read(path)?;
        Ok(self.set_bytes(mime_for(path), bytes))
    }

    pub fn buffer(&self) -> Result<Vec<u8>> {
        if self.status.is_empty() {
            return Err(anyhow!("status is empty"));
        }
        let mut out = Vec::new();
        // start line
        out.extend_from_slice(format!("{} {}\r\n", self.version, self.status).as_bytes());

        // headers
        for (k, v) in &self.headers {
            out.extend_from_slice(format!("{k}: {v}\r\n").as_bytes());
        }
        out.extend_from_slice(format!("Content-Length: {}\r\n", self.body.len()).as_bytes());
        out.extend_from_slice(b"\r\n");
        // body
        out.extend_from_slice(&self.body);
        Ok(out)
    }
}

fn mime_for(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        _ => "application/octet-stream",
    }
}
