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
    #[instrument(skip(self, req), fields(uri=%req.uri(), method=%req.method()), err)]
    fn execute(&self, req: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, ClientError> {
        debug!(
            name: "sending_request",
            body_len=req.body().len(),
            "Sending Request",
        );
        let response = self.send(req)?;
        let status = response.status();
        debug!(
            name: "response_received",
            status=status.as_u16(),
            response_len=response.body().len(),
            is_error=status.is_client_error() || status.is_server_error(),
            "Response Received",
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
