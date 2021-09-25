//! Contains the [Client] trait for executing
//! [Endpoints][crate::endpoint::Endpoint].
use crate::errors::ClientError;
use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, Response};
use std::ops::RangeInclusive;

/// An array of HTTP response codes which indicate a successful response
pub const HTTP_SUCCESS_CODES: RangeInclusive<u16> = 200..=208;

/// Represents an HTTP client which is capable of executing
/// [Endpoints][crate::endpoint::Endpoint] by sending the [Request] generated
/// by the Endpoint and returning a [Response].
#[async_trait]
pub trait Client: Sync + Send {
    /// Sends the given [Request] and returns a [Response]. Implementations
    /// should consolidate all errors into the [ClientError] type.
    async fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Bytes>, ClientError>;

    /// Returns the base URL the client is configured with. This is used for
    /// creating the fully qualified URLs used when executing
    /// [Endpoints][crate::endpoint::Endpoint].
    fn base(&self) -> &str;

    /// This method provides a common interface to
    /// [Endpoints][crate::endpoint::Endpoint] for execution.
    async fn execute(&self, req: Request<Vec<u8>>) -> Result<Response<Bytes>, ClientError> {
        log::info!(
            "Client sending {} request to {} with {} bytes of data",
            req.method().to_string(),
            req.uri(),
            req.body().len(),
        );
        let response = self.send(req).await?;

        log::info!(
            "Client received {} response with {} bytes of body data",
            response.status().as_u16(),
            response.body().len()
        );

        // Check response
        if !HTTP_SUCCESS_CODES.contains(&response.status().as_u16()) {
            return Err(ClientError::ServerResponseError {
                code: response.status().as_u16(),
                content: String::from_utf8(response.body().to_vec()).ok(),
            });
        }

        // Parse response content
        Ok(response)
    }
}
