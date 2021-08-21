use crate::{client::Client, enums::RequestMethod, errors::ClientError};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Method,
};
use serde_json::Value;
use std::str::FromStr;
use url::Url;

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
/// use rustify::clients::reqwest::ReqwestClient;
/// use rustify::endpoint::Endpoint;
/// use rustify_derive::Endpoint;
/// use serde::Serialize;
///
/// #[derive(Debug, Endpoint, Serialize)]
/// #[endpoint(path = "my/endpoint")]
/// struct MyEndpoint {}
///
/// let client = ReqwestClient::default("http://myapi.com");
/// let endpoint = MyEndpoint {};
/// let result = endpoint.exec(&client);
/// ```
///
/// [1]: https://docs.rs/reqwest/latest/reqwest/blocking/struct.Client.html
pub struct ReqwestClient {
    pub http: reqwest::blocking::Client,
    pub base: String,
}

impl ReqwestClient {
    /// Creates a new instance of [ReqwestClient] using the provided parameters
    pub fn new(base: &str, http: reqwest::blocking::Client) -> Self {
        ReqwestClient {
            base: base.to_string(),
            http,
        }
    }

    /// Creates a new instance of [ReqwestClient] with a default instance of
    /// [reqwest::blocking::Client][1].
    ///
    /// [1]: https://docs.rs/reqwest/latest/reqwest/blocking/struct.Client.html
    pub fn default(base: &str) -> Self {
        ReqwestClient {
            base: base.to_string(),
            http: reqwest::blocking::Client::default(),
        }
    }

    fn build_request(
        &self,
        method: &RequestMethod,
        url: &Url,
        query: &[(String, Value)],
        headers: &[(String, String)],
        data: Vec<u8>,
    ) -> Result<reqwest::blocking::Request, ClientError> {
        let builder = match method {
            RequestMethod::DELETE => match data.is_empty() {
                false => self.http.delete(url.as_ref()).body(data),
                true => self.http.delete(url.as_ref()),
            },
            RequestMethod::GET => self.http.get(url.as_ref()),
            RequestMethod::HEAD => match data.is_empty() {
                false => self.http.head(url.as_ref()).body(data),
                true => self.http.head(url.as_ref()),
            },
            RequestMethod::LIST => match data.is_empty() {
                false => self
                    .http
                    .request(Method::from_str("LIST").unwrap(), url.as_ref())
                    .body(data),
                true => self
                    .http
                    .request(Method::from_str("LIST").unwrap(), url.as_ref()),
            },
            RequestMethod::POST => match data.is_empty() {
                false => self.http.post(url.as_ref()).body(data),
                true => self.http.post(url.as_ref()),
            },
        };

        let mut map = HeaderMap::new();
        headers.iter().for_each(|h| {
            map.insert(
                HeaderName::from_str(h.0.as_str()).unwrap(),
                HeaderValue::from_str(h.1.as_str()).unwrap(),
            );
        });

        let req = builder.query(query).headers(map).build().map_err(|e| {
            ClientError::RequestBuildError {
                source: Box::new(e),
                url: url.to_string(),
                method: method.clone(),
            }
        })?;
        Ok(req)
    }
}

impl Client for ReqwestClient {
    fn base(&self) -> &str {
        self.base.as_str()
    }

    fn send(&self, req: crate::client::Request) -> Result<crate::client::Response, ClientError> {
        let request =
            self.build_request(&req.method, &req.url, &req.query, &req.headers, req.body)?;

        let err_url = req.url;
        let err_method = req.method;
        let response = self
            .http
            .execute(request)
            .map_err(|e| ClientError::RequestError {
                source: Box::new(e),
                url: err_url.to_string(),
                method: err_method,
            })?;

        let url = response.url().clone();
        let status_code = response.status().as_u16();
        let body = response
            .bytes()
            .map_err(|e| ClientError::ResponseError {
                source: Box::new(e),
            })?
            .to_vec();
        Ok(crate::client::Response {
            url,
            code: status_code,
            body,
        })
    }
}
