use rustify::endpoint::Endpoint;
use rustify_derive::Endpoint;
use serde::Serialize;

#[derive(Debug, Endpoint, Serialize)]
#[endpoint]
struct Test {}

#[derive(Debug, Endpoint, Serialize)]
#[endpoint()]
struct TestTwo {}

fn main() {}
