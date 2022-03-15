//! Contains the blocking [Client] trait for executing
//! [Endpoints][crate::endpoint::Endpoint].
use crate::{client::HTTP_SUCCESS_CODES, errors::ClientError};
use http::{Request, Response};

/// Represents an HTTP client which is capable of executing
/// [Endpoints][crate::endpoint::Endpoint] by sending the [Request] generated
/// by the Endpoint and returning a [Response].
pub trait Client {
    /// Sends the given [Request] and returns a [Response]. Implementations
    /// should consolidate all errors into the [ClientError] type.
    fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, ClientError>;

    /// Returns the base URL the client is configured with. This is used for
    /// creating the fully qualified URLs used when executing
    /// [Endpoints][crate::endpoint::Endpoint].
    fn base(&self) -> &str;

    /// This method provides a common interface to
    /// [Endpoints][crate::endpoint::Endpoint] for execution.
    #[instrument(skip(self, req), err)]
    fn execute(&self, req: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, ClientError> {
        debug!(
            "Client sending {} request to {} with {} bytes of data",
            req.method().to_string(),
            req.uri(),
            req.body().len(),
        );
        let response = self.send(req)?;

        debug!(
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
