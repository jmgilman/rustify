use rustify::endpoint::Endpoint;
use rustify_derive::Endpoint;
use serde::Serialize;

#[derive(Debug, Endpoint, Serialize)]
#[endpoint(path = "test/path", method = "TEST")]
struct Test {}

fn main() {}
