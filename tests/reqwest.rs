mod common;

use common::TestServer;
use httpmock::prelude::*;
use reqwest::header::HeaderValue;
use rustify::{
    clients::reqwest::{MiddleWare, ReqwestClient},
    endpoint::Endpoint,
};
use rustify_derive::Endpoint;
use serde::Serialize;

#[test]
fn test_server_error() {
    #[derive(Debug, Endpoint, Serialize)]
    #[endpoint(path = "test/path")]
    struct Test {}

    let t = TestServer::default();
    let e = Test {};
    let m = t.server.mock(|when, then| {
        when.method(GET).path("/test/path");
        then.status(500);
    });
    let r = e.execute(&t.client);

    m.assert();
    assert!(r.is_err());
}

#[test]
fn test_middleware() {
    #[derive(Debug, Endpoint, Serialize)]
    #[endpoint(path = "test/path")]
    struct Test {}

    struct TestMiddleWare {
        value: String,
    }
    impl MiddleWare for TestMiddleWare {
        fn handle(&self, mut r: reqwest::blocking::Request) -> reqwest::blocking::Request {
            r.headers_mut().append(
                "Test-Header",
                HeaderValue::from_str(self.value.as_str()).unwrap(),
            );
            r
        }
    }

    let client = ReqwestClient::with_middleware(
        "",
        Box::new(TestMiddleWare {
            value: "test".to_string(),
        }),
    );
    let t = TestServer::with_client(client);
    let e = Test {};
    let m = t.server.mock(|when, then| {
        when.method(GET)
            .path("/test/path")
            .header("Test-Header", "test");
        then.status(200);
    });
    let r = e.execute(&t.client);

    m.assert();
    assert!(r.is_ok());
}
