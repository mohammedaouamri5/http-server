use anyhow::{Result, anyhow};
use core::fmt;
use log::{error, info};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};

mod constants;
mod http;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    let listener = TcpListener::bind("127.0.0.1:4221")?;

    for stream in listener.incoming() {
        let mut stream = stream?; // `mut` here because we write to it below

        let _req = http::Request::HTTPRequest::from_stream(&stream);
        match _req {
            Ok(req) => {
                info!("request: {:?}", req);
                let mut writer = BufWriter::new(stream);
                if req.url.eq("/index.html") {
                    writer.write_fmt(format_args!(
                        "{} {}{}{}",
                        constants::http::VERSION,
                        constants::response::OK,
                        constants::http::CRLF,
                        constants::http::CRLF,
                    ))?;
                } else {
                    writer.write_fmt(format_args!(
                        "{} {}{}{}",
                        constants::http::VERSION,
                        constants::response::BAD_REQUEST,
                        constants::http::CRLF,
                        constants::http::CRLF,
                    ))?;
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
