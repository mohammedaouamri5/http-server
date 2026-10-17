use log::{debug, error, info, warn};
use std::io::Write;
use std::net::TcpListener;
use std::path::{Path, PathBuf};

use crate::constants::response::{INTERNAL_SERVER_ERROR, NOT_FOUND, OK};

mod constants;
mod http;
mod utile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_line_number(true)
        .format_module_path(true)
        .format_file(true)
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
                    let _path = utile::files::is_in_public(&req.url);
                    if _path.is_ok() {
                        let path = _path.unwrap();
                        if path.is_some() {
                            let path = path.unwrap();
                            res.set_status(OK);
                            res.set_file(path.to_str().unwrap())?;
                            let body = res.buffer().unwrap();
                            stream.write_all(&body)?;
                        }
                    }
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
