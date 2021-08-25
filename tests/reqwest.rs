/* mod common;

use common::TestServer;
use httpmock::prelude::*;
use rustify::endpoint::Endpoint;
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
    let r = e.exec(&t.client);

    m.assert();
    assert!(r.is_err());
} */
