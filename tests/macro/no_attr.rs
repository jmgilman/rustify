use rustify::endpoint::Endpoint;
use rustify_derive::Endpoint;
use serde::Serialize;

#[derive(Debug, Endpoint, Serialize)]
struct Test {}

fn main() {}
