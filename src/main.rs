use anyhow::{Result, anyhow};
use log::{debug, error, info, warn};
use std::io::Write;
use std::net::TcpListener;
use std::path::{Path, PathBuf};

use crate::constants::response::{INTERNAL_SERVER_ERROR, NOT_FOUND, OK};

mod constants;
mod http;

fn resolve_file(path: &str) -> Option<PathBuf> {
    let relative = if path == "/" {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };

    let base = Path::new("public").canonicalize().ok()?;
    let full = base.join(relative).canonicalize().ok()?;

    // Make sure the final path is still inside public/
    if full.starts_with(&base) && full.is_file() {
        Some(full)
    } else {
        None
    }
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    let listener = TcpListener::bind("127.0.0.1:4221")?;

    for stream in listener.incoming() {
        let mut stream = stream?; // `mut` here because we write to it below

        let _req = http::Request::HTTPRequest::from_stream(&stream);
        let mut res = http::Response::HTTPResponse::new();
        match _req {
            Ok(req) => {
                info!("request: {:?}", req);
                if req.method == "GET" {
                    if let Some(path) = req.url.strip_prefix("/") {
                        warn!("path: {}", path.to_string());
                        if let Some(path) = resolve_file(path) {
                            warn!("path: {}", path.to_string_lossy());
                            if res.set_file(&path.to_string_lossy()).is_err() {
                                res.set_status(INTERNAL_SERVER_ERROR);
                            }

                            res.set_status(OK);
                        } else {
                            res.set_status(NOT_FOUND);
                        }
                    } else {
                        res.set_status(NOT_FOUND);
                    }
                    let body = res.buffer().unwrap();
                    stream.write_all(&body)?;
                }
            }
            Err(e) => {
                error!("bad request: {}", e);
                stream.write_all(b"HTTP/1.1 500 Internal Server Error\r\n\r\n")?;
            }
        }
    }

    Ok(())
}
