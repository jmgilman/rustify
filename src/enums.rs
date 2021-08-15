/// Represents a HTTP request method
#[derive(Clone, Debug)]
pub enum RequestType {
    DELETE,
    GET,
    HEAD,
    LIST,
    POST,
}
