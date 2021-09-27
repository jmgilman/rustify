//! Contains an implementation of [Client][crate::client::Client] being backed
//! by the [reqwest](https://docs.rs/reqwest/) crate.

use crate::{client::Client as RustifyClient, errors::ClientError};
use async_trait::async_trait;
use http::{Request, Response};
use std::convert::TryFrom;

/// A client based on the
/// [reqwest::Client][1] which can be used for executing
/// [Endpoints][crate::endpoint::Endpoint]. A backing instance of a
/// [reqwest::Client][1] is used to increase performance and to save certain
/// characteristics across sessions. A base URL is required and is used to
/// qualify the full path of any [Endpoints][crate::endpoint::Endpoint] which
/// are executed by this client.
///
/// # Example
/// ```
/// use rustify::clients::reqwest::Client;
/// use rustify::Endpoint;
/// use rustify_derive::Endpoint;
/// use serde::Serialize;
///
/// #[derive(Debug, Endpoint, Serialize)]
/// #[endpoint(path = "my/endpoint")]
/// struct MyEndpoint {}
///
/// # tokio_test::block_on(async {
/// let client = Client::default("http://myapi.com");
/// let endpoint = MyEndpoint {};
/// let result = endpoint.exec(&client).await;
/// # })
/// ```
///
/// [1]: https://docs.rs/reqwest/latest/reqwest/struct.Client.html
pub struct Client {
    pub http: reqwest::Client,
    pub base: String,
}

impl Client {
    /// Creates a new instance of [Client] using the provided parameters.
    pub fn new(base: &str, http: reqwest::Client) -> Self {
        Client {
            base: base.to_string(),
            http,
        }
    }

    /// Creates a new instance of [Client] with a default instance of
    /// [reqwest::Client][1].
    ///
    /// [1]: https://docs.rs/reqwest/latest/reqwest/struct.Client.html
    pub fn default(base: &str) -> Self {
        Client {
            base: base.to_string(),
            http: reqwest::Client::default(),
        }
    }
}

#[async_trait]
impl RustifyClient for Client {
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
        let mut headers = http::header::HeaderMap::new();
        let http_resp = http::Response::builder().status(status_code);
        for v in response.headers().into_iter() {
            headers.append::<http::header::HeaderName>(v.0.into(), v.1.into());
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
