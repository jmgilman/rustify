use url::Url;

use crate::{enums::RequestType, errors::ClientError};

const HTTP_SUCCESS_CODES: [u16; 2] = [200, 204];
pub trait Client {
    fn send(&self, req: crate::client::Request) -> Result<crate::client::Response, ClientError>;

    fn base(&self) -> &str;

    fn execute(&self, req: Request) -> Result<Vec<u8>, ClientError> {
        let response = self.send(req)?;

        // Check response
        if !HTTP_SUCCESS_CODES.contains(&response.code) {
            return Err(ClientError::ServerResponseError {
                url: response.url.to_string(),
                code: response.code,
                content: response.content.clone(),
            });
        }

        // Parse response content
        Ok(response.content)
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub url: Url,
    pub method: RequestType,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub url: Url,
    pub code: u16,
    pub content: Vec<u8>,
}
