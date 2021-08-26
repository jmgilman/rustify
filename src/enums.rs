//! Contains common enums used across the crate

/// Represents a HTTP request method
#[derive(Clone, Debug)]
pub enum RequestMethod {
    DELETE,
    GET,
    HEAD,
    LIST,
    POST,
}

#[allow(clippy::from_over_into)]
impl Into<http::Method> for RequestMethod {
    fn into(self) -> http::Method {
        match self {
            RequestMethod::DELETE => http::Method::DELETE,
            RequestMethod::GET => http::Method::GET,
            RequestMethod::HEAD => http::Method::HEAD,
            RequestMethod::LIST => http::Method::from_bytes("LIST".as_bytes()).unwrap(),
            RequestMethod::POST => http::Method::POST,
        }
    }
}

/// Represents the type of a HTTP request body
#[derive(Clone, Debug)]
pub enum RequestType {
    JSON,
}

/// Represents the type of a HTTP response body
#[derive(Clone, Debug)]
pub enum ResponseType {
    JSON,
}
