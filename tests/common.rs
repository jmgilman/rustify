use bytes::Bytes;
use httpmock::prelude::*;
#[cfg(feature = "blocking")]
use rustify::blocking::clients::reqwest::Client as ReqwestBlocking;
use rustify::{
    clients::reqwest::Client as Reqwest,
    endpoint::{Endpoint, MiddleWare, Wrapper},
    errors::ClientError,
};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;

pub struct TestServer {
    pub server: MockServer,
    pub client: Reqwest,
}

#[cfg(feature = "blocking")]
pub struct TestServerBlocking {
    pub server: MockServer,
    pub client: ReqwestBlocking,
}

impl TestServer {
    #[allow(dead_code)]
    pub fn with_client(mut client: Reqwest) -> TestServer {
        let server = MockServer::start();
        let url = server.base_url();
        client.base = url;
        TestServer { server, client }
    }
}

impl Default for TestServer {
    fn default() -> Self {
        let server = MockServer::start();
        let url = server.base_url();
        TestServer {
            server,
            client: Reqwest::default(url.as_str()),
        }
    }
}

#[cfg(feature = "blocking")]
impl Default for TestServerBlocking {
    fn default() -> Self {
        let server = MockServer::start();
        let url = server.base_url();
        TestServerBlocking {
            server,
            client: ReqwestBlocking::default(url.as_str()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TestResponse {
    pub age: u8,
}

#[derive(Debug, Deserialize)]
pub struct TestWrapper {
    pub result: Value,
}

#[derive(Debug, Deserialize)]
pub struct TestGenericWrapper<T> {
    pub result: T,
}

impl<T: DeserializeOwned> Wrapper for TestGenericWrapper<T> {
    type Value = T;
}

pub struct Middle {}

impl MiddleWare for Middle {
    fn request<E: Endpoint>(
        &self,
        _: &E,
        req: &mut http::Request<Bytes>,
    ) -> Result<(), ClientError> {
        req.headers_mut()
            .append("X-API-Token", http::HeaderValue::from_static("mytoken"));
        Ok(())
    }
    fn response<E: Endpoint>(
        &self,
        _: &E,
        resp: &mut http::Response<Bytes>,
    ) -> Result<(), ClientError> {
        let resp_body = resp.body().clone();
        let wrapper: TestWrapper =
            serde_json::from_slice(&resp_body).map_err(|e| ClientError::ResponseParseError {
                source: e.into(),
                content: String::from_utf8(resp_body.to_vec()).ok(),
            })?;
        let data = wrapper.result.to_string();
        *resp.body_mut() = bytes::Bytes::from(data);
        Ok(())
    }
}
