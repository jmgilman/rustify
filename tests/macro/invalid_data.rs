use rustify::endpoint::Endpoint;
use rustify_derive::Endpoint;
use serde::Serialize;

#[derive(Debug, Endpoint, Serialize)]
#[endpoint(path = "test/path")]
struct Test {
    pub name: String,
    #[endpoint(data)]
    pub data: String,
}

#[derive(Debug, Endpoint, Serialize)]
#[endpoint(path = "test/path")]
struct TestTwo {
    pub name: String,
    #[endpoint(data)]
    pub data: Vec<u8>,
    #[endpoint(data)]
    pub data_two: Vec<u8>,
}

fn main() {}
