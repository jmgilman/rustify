use crate::{enums::RequestType, errors::ClientError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use url::Url;

pub trait Endpoint: Debug + Sized {
    type RequestData: Serialize;
    type Response: DeserializeOwned;

    fn action(&self) -> String;
    fn method(&self) -> RequestType;
    fn data(&self) -> Option<&Self::RequestData>;

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
        client.execute(self)
    }
}

#[derive(Deserialize, Debug)]
pub struct EmptyEndpointResult {}

#[derive(serde::Serialize, Debug)]
pub struct EmptyEndpointData {}
