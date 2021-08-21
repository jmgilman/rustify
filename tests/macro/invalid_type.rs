use rustify::endpoint::Endpoint;
use rustify_derive::Endpoint;
use serde::Serialize;

#[derive(Debug, Endpoint, Serialize)]
#[endpoint(path = "test/path", request_type = "BAD", response_type = "BAD")]
struct Test {}

fn main() {}
