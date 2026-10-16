pub mod response {
    pub const OK: &str = "200 OK";
    pub const BAD_REQUEST: &str = "400 Bad Request";
    pub const NOT_FOUND: &str = "404 Not Found";
    pub const METHOD_NOT_ALLOWED: &str = "405 Method Not Allowed";
    pub const REQUEST_TIMEOUT: &str = "408 Request Timeout";
    pub const PAYLOAD_TOO_LARGE: &str = "413 Payload Too Large";
    pub const HEADER_TOO_LARGE: &str = "431 Request Header Fields Too Large";
    pub const INTERNAL_SERVER_ERROR: &str = "500 Internal Server Error";
    pub const FORBIDDEN: &str = "403 Forbidden";
}

pub mod server {
    pub const ADDRESS: &str = "127.0.0.1:4221";
    pub const BUFFER_SIZE: usize = 1024;
}

pub mod http {
    pub const VERSION: &str = "HTTP/1.1";
    pub const CRLF: &str = "\r\n";
}
