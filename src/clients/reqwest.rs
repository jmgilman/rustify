use std::str::FromStr;

use reqwest::Method;
use url::Url;

use crate::{client::Client, enums::RequestType, errors::ClientError};
type MiddleWare = Box<dyn Fn(reqwest::blocking::Request) -> reqwest::blocking::Request>;
pub struct ReqwestClient {
    pub http: reqwest::blocking::Client,
    pub base: String,
    pub middle: MiddleWare,
}

impl ReqwestClient {
    pub fn new(base: &str, http: reqwest::blocking::Client, middle: MiddleWare) -> Self {
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
            middle: Box::new(|r| r),
        }
    }

    fn build_request(
        &self,
        method: &RequestType,
        url: &Url,
        data: String,
    ) -> Result<reqwest::blocking::Request, ClientError> {
        let data = match data.as_str() {
            "null" => None,
            "{}" => None,
            _ => Some(data),
        };
        let builder = match method {
            RequestType::DELETE => match data {
                Some(d) => self.http.delete(url.as_ref()).body(d),
                None => self.http.delete(url.as_ref()),
            },
            RequestType::GET => self.http.get(url.as_ref()),
            RequestType::HEAD => match data {
                Some(d) => self.http.head(url.as_ref()).body(d),
                None => self.http.head(url.as_ref()),
            },
            RequestType::LIST => match data {
                Some(d) => self
                    .http
                    .request(Method::from_str("LIST").unwrap(), url.as_ref())
                    .body(d),
                None => self
                    .http
                    .request(Method::from_str("LIST").unwrap(), url.as_ref()),
            },
            RequestType::POST => match data {
                Some(d) => self.http.post(url.as_ref()).body(d),
                None => self.http.post(url.as_ref()),
            },
        };
        let req = builder
            .build()
            .map_err(|e| ClientError::RequestBuildError {
                source: Box::new(e),
                url: url.to_string(),
                method: method.clone(),
            })?;
        Ok((self.middle)(req))
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
        let content = response.text().map_err(|e| ClientError::ResponseError {
            source: Box::new(e),
        })?;
        Ok(crate::client::Response {
            url,
            code: status_code,
            content,
        })
    }
}
