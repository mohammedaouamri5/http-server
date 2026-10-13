pub mod response {
    pub const INTERNAL_SERVER_ERROR: &str = "500 Internal Server Error";
    pub const NOT_FOUND: &str = "404 Not Found";
    pub const BAD_REQUEST: &str = "400 Bad Request";
    pub const OK: &str = "200 OK";
}

pub mod server {
    pub const ADDRESS: &str = "127.0.0.1:4221";
    pub const BUFFER_SIZE: usize = 1024;
}

pub mod http {
    pub const VERSION: &str = "HTTP/1.1";
    pub const CRLF: &str = "\r\n";
}
