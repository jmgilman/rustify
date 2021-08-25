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

#[derive(Clone, Debug)]
pub enum RequestType {
    JSON,
}

#[derive(Clone, Debug)]
pub enum ResponseType {
    JSON,
}
