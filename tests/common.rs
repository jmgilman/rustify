use httpmock::prelude::*;
use rustify::{
    clients::reqwest::ReqwestClient,
    endpoint::{Endpoint, MiddleWare},
    errors::ClientError,
};
use serde::Deserialize;
use serde_json::Value;

pub struct TestServer {
    pub server: MockServer,
    pub client: ReqwestClient,
}

impl TestServer {
    #[allow(dead_code)]
    pub fn with_client(mut client: ReqwestClient) -> TestServer {
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
            client: ReqwestClient::default(url.as_str()),
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

pub struct Middle {}
impl MiddleWare for Middle {
    fn request<E: Endpoint>(
        &self,
        _: &E,
        req: &mut rustify::client::Request,
    ) -> Result<(), ClientError> {
        req.headers
            .push(("X-API-Token".to_string(), "mytoken".to_string()));
        Ok(())
    }
    fn response<E: Endpoint>(
        &self,
        _: &E,
        resp: &mut rustify::client::Response,
    ) -> Result<(), ClientError> {
        let err_body = resp.body.clone();
        let wrapper: TestWrapper =
            serde_json::from_slice(&resp.body).map_err(|e| ClientError::ResponseParseError {
                source: Box::new(e),
                content: String::from_utf8(err_body).ok(),
            })?;
        resp.body = wrapper.result.to_string().as_bytes().to_vec();
        Ok(())
    }
}
