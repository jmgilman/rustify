mod common;

use common::TestServer;
use derive_builder::Builder;
use httpmock::prelude::*;
use rustify::{endpoint::Endpoint, errors::ClientError};
use rustify_derive::Endpoint;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_with::skip_serializing_none;
use test_env_log::test;

#[test]
fn test_path() {
    #[derive(Debug, Endpoint, Serialize)]
    #[endpoint(path = "test/path")]
    struct Test {}

    let t = TestServer::default();
    let e = Test {};
    let m = t.server.mock(|when, then| {
        when.method(GET).path("/test/path");
        then.status(200);
    });
    let r = e.execute(&t.client);

    m.assert();
    assert!(r.is_ok());
}

#[test]
fn test_path_method() {
    #[derive(Debug, Endpoint, Serialize)]
    #[endpoint(path = "test/path", method = "RequestType::POST")]
    struct Test {}

    let t = TestServer::default();
    let e = Test {};
    let m = t.server.mock(|when, then| {
        when.method(POST).path("/test/path");
        then.status(200);
    });
    let r = e.execute(&t.client);

    m.assert();
    assert!(r.is_ok());
}

#[test]
fn test_path_method_with_format() {
    #[derive(Debug, Endpoint, Serialize)]
    #[endpoint(path = "test/path/{self.name}", method = "RequestType::POST")]
    struct Test {
        #[serde(skip)]
        name: String,
    }

    let t = TestServer::default();
    let e = Test {
        name: "test".to_string(),
    };
    let m = t.server.mock(|when, then| {
        when.method(POST).path("/test/path/test").body("");
        then.status(200);
    });
    let r = e.execute(&t.client);

    m.assert();
    assert!(r.is_ok());
}

#[test]
fn test_path_method_with_data() {
    #[derive(Debug, Endpoint, Serialize)]
    #[endpoint(path = "test/path", method = "RequestType::POST")]
    struct Test {
        name: String,
    }

    let t = TestServer::default();
    let e = Test {
        name: "test".to_string(),
    };
    let m = t.server.mock(|when, then| {
        when.method(POST)
            .path("/test/path")
            .json_body(json!({ "name": "test" }));
        then.status(200);
    });
    let r = e.execute(&t.client);

    m.assert();
    assert!(r.is_ok());
}

#[test]
fn test_path_result() {
    #[derive(Debug, Endpoint, Serialize)]
    #[endpoint(path = "test/path", result = "TestResponse")]
    struct Test {}

    #[derive(Deserialize)]
    struct TestResponse {
        age: u8,
    }

    let t = TestServer::default();
    let e = Test {};
    let m = t.server.mock(|when, then| {
        when.method(GET).path("/test/path");
        then.status(200).json_body(json!({"age": 30}));
    });
    let r = e.execute(&t.client);

    m.assert();
    assert!(r.is_ok());
    assert_eq!(r.unwrap().unwrap().age, 30);
}

#[test]
fn test_builder() {
    #[derive(Builder, Debug, Endpoint, Serialize)]
    #[endpoint(path = "test/path", method = "RequestType::POST", builder = "true")]
    #[builder(setter(into))]
    struct Test {
        name: String,
    }

    let t = TestServer::default();
    let m = t.server.mock(|when, then| {
        when.method(POST)
            .path("/test/path")
            .json_body(json!({ "name": "test" }));
        then.status(200).json_body(json!({"age": 30}));
    });
    let r = Test::builder().name("test").execute(&t.client);

    m.assert();
    assert!(r.is_ok());
}

#[test]
fn test_transform() {
    #[derive(Debug, Endpoint, Serialize)]
    #[endpoint(path = "test/path", result = "TestResponse", transform = "strip")]
    struct Test {}

    #[derive(Debug, Serialize, Deserialize)]
    struct TestResponse {
        age: u8,
    }

    #[derive(Debug, Deserialize)]
    struct TestWrapper {
        result: TestResponse,
    }

    fn strip(res: String) -> Result<String, ClientError> {
        let r: TestWrapper =
            serde_json::from_str(res.as_str()).map_err(|e| ClientError::GenericError {
                source: Box::new(e),
            })?;
        serde_json::to_string(&r.result).map_err(|e| ClientError::GenericError {
            source: Box::new(e),
        })
    }

    let t = TestServer::default();
    let e = Test {};
    let m = t.server.mock(|when, then| {
        when.method(GET).path("/test/path");
        then.status(200).json_body(json!({"result": {"age": 30}}));
    });
    let r = e.execute(&t.client);

    m.assert();
    assert!(r.is_ok());
    assert_eq!(r.unwrap().unwrap().age, 30);
}

#[test]
fn test_complex() {
    #[skip_serializing_none]
    #[derive(Builder, Debug, Default, Endpoint, Serialize)]
    #[endpoint(
        path = "test/path/{self.name}",
        method = "RequestType::POST",
        result = "TestResponse",
        transform = "strip",
        builder = "true"
    )]
    #[builder(setter(into, strip_option), default)]
    struct Test {
        #[serde(skip)]
        name: String,
        kind: String,
        special: Option<bool>,
        optional: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct TestResponse {
        age: u8,
    }

    #[derive(Debug, Deserialize)]
    struct TestWrapper {
        result: TestResponse,
    }

    fn strip(res: String) -> Result<String, ClientError> {
        let r: TestWrapper =
            serde_json::from_str(res.as_str()).map_err(|e| ClientError::GenericError {
                source: Box::new(e),
            })?;
        serde_json::to_string(&r.result).map_err(|e| ClientError::GenericError {
            source: Box::new(e),
        })
    }

    let t = TestServer::default();
    let m = t.server.mock(|when, then| {
        when.method(POST)
            .path("/test/path/test")
            .json_body(json!({ "kind": "test" }));
        then.status(200).json_body(json!({"result": {"age": 30}}));
    });
    let r = Test::builder().name("test").kind("test").execute(&t.client);

    m.assert();
    assert!(r.is_ok());
    assert_eq!(r.unwrap().unwrap().age, 30);
}
