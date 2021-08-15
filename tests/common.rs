use httpmock::prelude::*;
use rustify::clients::reqwest::ReqwestClient;

pub struct TestServer {
    pub server: MockServer,
    pub client: ReqwestClient,
}

impl TestServer {
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
