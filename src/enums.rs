//! Contains common enums used across the crate

use http::HeaderValue;

/// Represents a HTTP request method
#[derive(Clone, Debug)]
pub enum RequestMethod {
    CONNECT,
    DELETE,
    GET,
    HEAD,
    LIST,
    OPTIONS,
    PATCH,
    POST,
    PUT,
    TRACE,
}

#[allow(clippy::from_over_into)]
impl Into<http::Method> for RequestMethod {
    fn into(self) -> http::Method {
        match self {
            RequestMethod::CONNECT => http::Method::CONNECT,
            RequestMethod::DELETE => http::Method::DELETE,
            RequestMethod::GET => http::Method::GET,
            RequestMethod::HEAD => http::Method::HEAD,
            RequestMethod::LIST => http::Method::from_bytes("LIST".as_bytes()).unwrap(),
            RequestMethod::OPTIONS => http::Method::OPTIONS,
            RequestMethod::PATCH => http::Method::PATCH,
            RequestMethod::POST => http::Method::POST,
            RequestMethod::PUT => http::Method::PUT,
            RequestMethod::TRACE => http::Method::TRACE,
        }
    }
}

/// Represents the type of a HTTP request body
#[derive(Clone, Debug)]
pub enum RequestType {
    JSON,
}

impl From<RequestType> for HeaderValue {
    fn from(ty: RequestType) -> Self {
        match ty {
            RequestType::JSON => HeaderValue::from_static("application/json"),
        }
    }
}

/// Represents the type of a HTTP response body
#[derive(Clone, Debug)]
pub enum ResponseType {
    JSON,
}
