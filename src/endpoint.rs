use crate::{client::Request, enums::RequestType, errors::ClientError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use url::Url;

pub trait Endpoint: Debug + Serialize + Sized {
    type Response: DeserializeOwned;

    fn action(&self) -> String;
    fn method(&self) -> RequestType;

    fn build_url(&self, base: &str) -> Result<url::Url, ClientError> {
        let mut url = Url::parse(base).map_err(|e| ClientError::UrlParseError {
            url: base.to_string(),
            source: e,
        })?;
        url.path_segments_mut()
            .unwrap()
            .extend(self.action().split("/"));
        Ok(url)
    }

    fn execute<C: crate::client::Client>(
        &self,
        client: &C,
    ) -> Result<Option<Self::Response>, ClientError> {
        let url = self.build_url(client.base())?;
        let method = self.method();
        let data = serde_json::to_string(self).map_err(|e| ClientError::DataParseError {
            source: Box::new(e),
        })?;
        self.parse(client.execute(Request { url, method, data }))
    }

    fn parse(
        &self,
        res: Result<String, ClientError>,
    ) -> Result<Option<Self::Response>, ClientError> {
        match res {
            Ok(r) => match r.is_empty() {
                false => Ok(Some(serde_json::from_str(r.as_str()).map_err(|e| {
                    ClientError::ResponseParseError {
                        source: Box::new(e),
                        content: r.clone(),
                    }
                })?)),
                true => Ok(None),
            },
            Err(e) => Err(e),
        }
    }

    fn transform(&self, res: String) -> Result<String, ClientError> {
        Ok(res)
    }
}

#[derive(Deserialize, Debug)]
pub struct EmptyEndpointResult {}

#[derive(serde::Serialize, Debug)]
pub struct EmptyEndpointData {}
