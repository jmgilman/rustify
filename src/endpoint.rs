use crate::{client::Request, enums::RequestType, errors::ClientError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use url::Url;

pub trait Endpoint: Debug + Serialize + Sized {
    type Response: DeserializeOwned;

    fn action(&self) -> String;
    fn method(&self) -> RequestType;

    fn build_url(&self, base: &str) -> Result<url::Url, ClientError> {
        log::info!(
            "Building endpoint url from {} base URL and {} action",
            base,
            self.action()
        );

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
        log::info!("Executing endpoint");
        log::debug! {"Endpoint: {:#?}", self};

        let url = self.build_url(client.base())?;
        let method = self.method();
        let data = serde_json::to_string(self).map_err(|e| ClientError::DataParseError {
            source: Box::new(e),
        })?;
        let data = match data.as_str() {
            "null" => "".to_string(),
            "{}" => "".to_string(),
            _ => data,
        }
        .into_bytes();
        self.parse(client.execute(Request { url, method, data }))
    }

    fn parse(
        &self,
        res: Result<Vec<u8>, ClientError>,
    ) -> Result<Option<Self::Response>, ClientError> {
        match res {
            Ok(r) => {
                let r_conv_err = r.clone();
                let r_parse_err = r.clone();
                let c = String::from_utf8(r).map_err(|e| ClientError::ResponseParseError {
                    source: Box::new(e),
                    content: r_conv_err,
                })?;

                log::info!("Parsing JSON result from string");
                log::debug!("Content before transform: {}", c);
                let c = self.transform(c)?;
                log::debug!("Content after transform: {}", c);
                match c.is_empty() {
                    false => Ok(Some(serde_json::from_str(c.as_str()).map_err(|e| {
                        ClientError::ResponseParseError {
                            source: Box::new(e),
                            content: r_parse_err,
                        }
                    })?)),
                    true => Ok(None),
                }
            }
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
