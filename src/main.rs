use anyhow::Result;
use env_logger::Env;
use log::{debug, error, info, warn};
use std::io::Write;
use std::net::TcpListener;

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .format_module_path(true)
        .format_source_path(true)
        .init();


    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                info!("Connection established");
                const BUFF: &[u8] = b"HTTP/1.1 200 OK\r\n\r\nHello world!\r\n";
                _stream.write(BUFF)?;
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    Ok(())
}
