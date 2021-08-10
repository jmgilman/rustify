use serde::{de::DeserializeOwned, Serialize};
use url::Url;

use crate::{endpoint::Endpoint, enums::RequestType, errors::ClientError};

const HTTP_SUCCESS_CODES: [u16; 2] = [200, 204];
pub trait Client {
    fn send<S: Serialize>(
        &self,
        req: crate::client::Request<S>,
    ) -> Result<crate::client::Response, ClientError>;

    fn base(&self) -> &str;

    fn execute<E: Endpoint, D: DeserializeOwned>(
        &self,
        endpoint: &E,
    ) -> Result<Option<D>, ClientError> {
        let url = endpoint.build_url(self.base())?;
        let method = endpoint.method();
        let data = endpoint.data();
        let response = self.send(crate::client::Request { url, method, data })?;

        // Check response
        if !HTTP_SUCCESS_CODES.contains(&response.code) {
            return Err(ClientError::ServerResponseError {
                url: response.url.to_string(),
                code: response.code,
                content: response.content.clone(),
            });
        }

        // Check for response content
        if response.content.is_empty() {
            return Ok(None);
        }

        // Parse response content
        serde_json::from_str(response.content.as_str()).map_err(|e| {
            ClientError::ResponseParseError {
                source: Box::new(e),
                content: response.content.clone(),
            }
        })
    }
}

pub struct Request<'a, S: Serialize> {
    pub url: Url,
    pub method: RequestType,
    pub data: Option<&'a S>,
}

pub struct Response {
    pub url: Url,
    pub code: u16,
    pub content: String,
}
