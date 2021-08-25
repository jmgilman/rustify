use crate::{client::Client as RustifyClient, errors::ClientError};
use async_trait::async_trait;
use bytes::Bytes;
use http::{Request, Response};
use std::convert::TryFrom;

/// A client based on the
/// [reqwest::blocking::Client][1] which can be used for executing
/// [Endpoints][crate::endpoint::Endpoint]. A backing instance of a
/// [reqwest::blocking::Client][1] is used to increase performance and save
/// certain characteristics across sessions. A base URL is required and is used
/// to qualify the full path of any [Endpoints][crate::endpoint::Endpoint] which
/// are executed by this client.
///
/// # Example
/// ```
/// use rustify::clients::reqwest::Client;
/// use rustify::endpoint::Endpoint;
/// use rustify_derive::Endpoint;
/// use serde::Serialize;
///
/// #[derive(Debug, Endpoint, Serialize)]
/// #[endpoint(path = "my/endpoint")]
/// struct MyEndpoint {}
///
/// let client = Client::default("http://myapi.com");
/// let endpoint = MyEndpoint {};
/// let result = endpoint.exec(&client);
/// ```
///
/// [1]: https://docs.rs/reqwest/latest/reqwest/blocking/struct.Client.html
pub struct Client {
    pub http: reqwest::Client,
    pub base: String,
}

impl Client {
    /// Creates a new instance of [ReqwestClient] using the provided parameters
    pub fn new(base: &str, http: reqwest::Client) -> Self {
        Client {
            base: base.to_string(),
            http,
        }
    }

    /// Creates a new instance of [ReqwestClient] with a default instance of
    /// [reqwest::blocking::Client][1].
    ///
    /// [1]: https://docs.rs/reqwest/latest/reqwest/blocking/struct.Client.html
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

    async fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Bytes>, ClientError> {
        let request = reqwest::Request::try_from(req).unwrap();

        let url_err = request.url().to_string();
        let method_err = request.method().to_string();
        let response = self
            .http
            .execute(request)
            .await
            .map_err(|e| ClientError::RequestError {
                source: Box::new(e),
                url: url_err,
                method: method_err,
            })?;

        let status_code = response.status().as_u16();
        let mut headers = http::header::HeaderMap::new();
        let http_resp = http::Response::builder().status(status_code);
        for v in response.headers().into_iter() {
            headers.append::<http::header::HeaderName>(v.0.into(), v.1.into());
        }
        Ok(http_resp
            .body(
                response
                    .bytes()
                    .await
                    .map_err(|e| ClientError::ResponseError {
                        source: Box::new(e),
                    })?,
            )
            .unwrap())
    }
}
