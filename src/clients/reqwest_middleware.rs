//! Contains an implementation of [Client][crate::client::Client] being backed
//! by the [reqwest](https://docs.rs/reqwest/) crate.

use crate::{client::Client as RustifyClient, errors::ClientError};
use async_trait::async_trait;
use http::{Request, Response};
use std::convert::TryFrom;

/// A client based on the
/// [reqwest_middleware::ClientWithMiddleware][1] which can be used for executing
/// [Endpoints][crate::endpoint::Endpoint]. A backing instance of a
/// [reqwest_middleware::ClientWithMiddleware][1] is used to increase performance and to save certain
/// characteristics across sessions. A base URL is required and is used to
/// qualify the full path of any [Endpoints][crate::endpoint::Endpoint] which
/// are executed by this client.
///
/// # Example
/// ```
/// use rustify::clients::reqwest_middleware::ClientWithMiddleware;
/// use rustify::Endpoint;
/// use rustify_derive::Endpoint;
/// use serde::Serialize;
///
/// #[derive(Debug, Endpoint, Serialize)]
/// #[endpoint(path = "my/endpoint")]
/// struct MyEndpoint {}
///
/// # tokio_test::block_on(async {
/// let client = ClientWithMiddleware::default("http://myapi.com");
/// let endpoint = MyEndpoint {};
/// let result = endpoint.exec(&client).await;
/// # })
/// ```
///
/// [1]: https://docs.rs/reqwest-middleware/latest/reqwest_middleware/struct.ClientWithMiddleware.html
pub struct ClientWithMiddleware {
    pub http: reqwest_middleware::ClientWithMiddleware,
    pub base: String,
}

impl ClientWithMiddleware {
    /// Creates a new instance of [ClientWithMiddleware] using the provided parameters.
    pub fn new(base: &str, http: reqwest_middleware::ClientWithMiddleware) -> Self {
        Self {
            base: base.to_string(),
            http,
        }
    }

    /// Creates a new instance of [ClientWithMiddleware] with a default instance of
    /// [reqwest_middleware::ClientWithMiddleware][1].
    ///
    /// [1]: https://docs.rs/reqwest-middleware/latest/reqwest_middleware/struct.ClientWithMiddleware.html
    pub fn default(base: &str) -> Self {
        Self {
            base: base.to_string(),
            http: reqwest_middleware::ClientBuilder::new(reqwest::Client::default()).build(),
        }
    }
}

#[async_trait]
impl RustifyClient for ClientWithMiddleware {
    fn base(&self) -> &str {
        self.base.as_str()
    }

    #[instrument(skip(self, req), err)]
    async fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, ClientError> {
        let request = reqwest::Request::try_from(req)
            .map_err(|e| ClientError::ReqwestBuildError { source: e })?;

        let url_err = request.url().to_string();
        let method_err = request.method().to_string();
        let response = self
            .http
            .execute(request)
            .await
            .map_err(|e| ClientError::RequestError {
                source: e.into(),
                url: url_err,
                method: method_err,
            })?;

        let status_code = response.status().as_u16();
        let mut http_resp = http::Response::builder().status(status_code);
        for v in response.headers().into_iter() {
            http_resp = http_resp.header(v.0, v.1);
        }

        http_resp
            .body(
                response
                    .bytes()
                    .await
                    .map_err(|e| ClientError::ResponseError { source: e.into() })?
                    .to_vec(),
            )
            .map_err(|e| ClientError::ResponseError { source: e.into() })
    }
}
