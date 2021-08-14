use std::str::FromStr;

use reqwest::Method;
use url::Url;

pub trait MiddleWare {
    fn handle(&self, r: reqwest::blocking::Request) -> reqwest::blocking::Request;
}

struct DefaultMiddleWare {}
impl MiddleWare for DefaultMiddleWare {
    fn handle(&self, r: reqwest::blocking::Request) -> reqwest::blocking::Request {
        r
    }
}

use crate::{client::Client, enums::RequestType, errors::ClientError};
pub struct ReqwestClient {
    pub http: reqwest::blocking::Client,
    pub base: String,
    pub middle: Box<dyn MiddleWare>,
}

impl ReqwestClient {
    pub fn new(base: &str, http: reqwest::blocking::Client, middle: Box<dyn MiddleWare>) -> Self {
        ReqwestClient {
            base: base.to_string(),
            http,
            middle,
        }
    }

    pub fn default(base: &str) -> Self {
        ReqwestClient {
            base: base.to_string(),
            http: reqwest::blocking::Client::default(),
            middle: Box::new(DefaultMiddleWare {}),
        }
    }

    pub fn with_middleware(base: &str, middle: Box<dyn MiddleWare>) -> ReqwestClient {
        ReqwestClient {
            base: base.to_string(),
            http: reqwest::blocking::Client::default(),
            middle: middle,
        }
    }

    fn build_request(
        &self,
        method: &RequestType,
        url: &Url,
        data: Vec<u8>,
    ) -> Result<reqwest::blocking::Request, ClientError> {
        let builder = match method {
            RequestType::DELETE => match data.is_empty() {
                false => self.http.delete(url.as_ref()).body(data),
                true => self.http.delete(url.as_ref()),
            },
            RequestType::GET => self.http.get(url.as_ref()),
            RequestType::HEAD => match data.is_empty() {
                false => self.http.head(url.as_ref()).body(data),
                true => self.http.head(url.as_ref()),
            },
            RequestType::LIST => match data.is_empty() {
                false => self
                    .http
                    .request(Method::from_str("LIST").unwrap(), url.as_ref())
                    .body(data),
                true => self
                    .http
                    .request(Method::from_str("LIST").unwrap(), url.as_ref()),
            },
            RequestType::POST => match data.is_empty() {
                false => self.http.post(url.as_ref()).body(data),
                true => self.http.post(url.as_ref()),
            },
        };
        let req = builder
            .build()
            .map_err(|e| ClientError::RequestBuildError {
                source: Box::new(e),
                url: url.to_string(),
                method: method.clone(),
            })?;
        Ok(self.middle.handle(req))
    }
}

impl Client for ReqwestClient {
    fn base(&self) -> &str {
        self.base.as_str()
    }

    fn send(&self, req: crate::client::Request) -> Result<crate::client::Response, ClientError> {
        let request = self.build_request(&req.method, &req.url, req.data.clone())?;
        let response = self
            .http
            .execute(request)
            .map_err(|e| ClientError::RequestError {
                source: Box::new(e),
                request: req.clone(),
            })?;

        let url = response.url().clone();
        let status_code = response.status().as_u16();
        let content = response
            .bytes()
            .map_err(|e| ClientError::ResponseError {
                source: Box::new(e),
            })?
            .to_vec();
        Ok(crate::client::Response {
            url,
            code: status_code,
            content,
        })
    }
}
