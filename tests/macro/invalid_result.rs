use rustify::endpoint::Endpoint;
use rustify_derive::Endpoint;
use serde::Serialize;

#[derive(Debug, Endpoint, Serialize)]
#[endpoint(path = "test/path", result = "DoesNotExist")]
struct Test {}

fn main() {}
