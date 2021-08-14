use httpmock::prelude::*;
use rustify::clients::reqwest::ReqwestClient;

pub struct TestServer {
    pub server: MockServer,
    pub client: ReqwestClient,
}

impl TestServer {
    pub fn new() -> TestServer {
        let server = MockServer::start();
        let url = server.base_url().clone();
        TestServer {
            server,
            client: ReqwestClient::default(url.as_str()),
        }
    }

    pub fn with_client(mut client: ReqwestClient) -> TestServer {
        let server = MockServer::start();
        let url = server.base_url().clone();
        client.base = url;
        TestServer { server, client }
    }
}
