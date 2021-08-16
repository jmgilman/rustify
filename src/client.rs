use crate::{enums::RequestType, errors::ClientError};
use std::ops::RangeInclusive;
use url::Url;

/// An array of HTTP response codes which indicate a successful response
const HTTP_SUCCESS_CODES: RangeInclusive<u16> = 200..=208;

/// Represents an HTTP client which is capable of executing
/// [Endpoints][crate::endpoint::Endpoint] by sending the [Request] generated
/// by the Endpoint and returning a [Response].
pub trait Client {
    /// Sends the given [Request] and returns a [Response]. Implementations
    /// should consolidate all errors into the [ClientError] type.
    fn send(&self, req: crate::client::Request) -> Result<crate::client::Response, ClientError>;

    /// Returns the base URL the client is configured with. This is used for
    /// creating the fully qualified URLs used when executing
    /// [Endpoints][crate::endpoint::Endpoint].
    fn base(&self) -> &str;

    /// This method provides a common interface to
    /// [Endpoints][crate::endpoint::Endpoint] for execution.
    fn execute(&self, req: Request) -> Result<Vec<u8>, ClientError> {
        log::info!(
            "Client sending {:#?} request to {} with {} bytes of data",
            req.method,
            req.url,
            req.data.len()
        );
        let response = self.send(req)?;

        log::info!(
            "Client received {} response from {} with {} bytes of body data",
            response.code,
            response.url,
            response.content.len()
        );

        // Check response
        if !HTTP_SUCCESS_CODES.contains(&response.code) {
            return Err(ClientError::ServerResponseError {
                url: response.url.to_string(),
                code: response.code,
                content: String::from_utf8(response.content).ok(),
            });
        }

        // Parse response content
        Ok(response.content)
    }
}

/// Represents an HTTP request
#[derive(Debug, Clone)]
pub struct Request {
    pub url: Url,
    pub method: RequestType,
    pub data: Vec<u8>,
}

/// Represents an HTTP response
#[derive(Debug, Clone)]
pub struct Response {
    pub url: Url,
    pub code: u16,
    pub content: Vec<u8>,
}
