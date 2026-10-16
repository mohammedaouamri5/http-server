use std::fmt;
use std::io::{self, BufRead, BufReader, Read};
use std::net::TcpStream;

use crate::constants::response::{
    BAD_REQUEST, FORBIDDEN, HEADER_TOO_LARGE, INTERNAL_SERVER_ERROR, METHOD_NOT_ALLOWED, NOT_FOUND,
    PAYLOAD_TOO_LARGE, REQUEST_TIMEOUT,
};

const MAX_LINE: u64 = 8 * 1024;
const MAX_HEADERS: usize = 100;
const MAX_BODY: usize = 1024 * 1024;

#[derive(Debug)]
pub enum HTTPError {
    BadRequest(String),
    NotFound(String),
    MethodNotAllowed(String),
    Timeout(String),
    BodyTooLarge(String),
    HeaderTooLarge(String),
    InternalServerError(String),
    Forbidden(String),
    Io(io::Error),
}

impl HTTPError {
    /// Status line to send to the client, or None if no response is
    /// possible (the socket itself is broken).
    pub fn status_line(&self) -> Option<&'static str> {
        match self {
            HTTPError::BadRequest(_) => Some(BAD_REQUEST),
            HTTPError::NotFound(_) => Some(NOT_FOUND),
            HTTPError::MethodNotAllowed(_) => Some(METHOD_NOT_ALLOWED),
            HTTPError::Timeout(_) => Some(REQUEST_TIMEOUT),
            HTTPError::BodyTooLarge(_) => Some(PAYLOAD_TOO_LARGE),
            HTTPError::HeaderTooLarge(_) => Some(HEADER_TOO_LARGE),
            HTTPError::InternalServerError(_) => Some(INTERNAL_SERVER_ERROR),
            HTTPError::Forbidden(_) => Some(FORBIDDEN),
            HTTPError::Io(_) => None,
        }
    }
}

impl fmt::Display for HTTPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HTTPError::BadRequest(msg) => write!(f, "bad request: {msg}"),
            HTTPError::Io(e) => write!(f, "io error: {e}"),
            other => write!(f, "{}", other.status_line().unwrap_or("unknown error")),
        }
    }
}

impl std::error::Error for HTTPError {}

impl From<io::Error> for HTTPError {
    fn from(e: io::Error) -> Self {
        match e.kind() {
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => {
                HTTPError::Timeout(e.to_string())
            }
            _ => HTTPError::Io(e),
        }
    }
}

#[macro_export]
macro_rules! http_err {
    ($variant:ident, $($arg:tt)*) => {{
        let msg = format!($($arg)*);
        #[cfg(debug_assertions)]
        { HTTPError::$variant(format!("{} [{}:{}:{}]", msg, file!(), line!(), column!())) }
        #[cfg(not(debug_assertions))]
        { HTTPError::$variant(msg) }
    }};
}
