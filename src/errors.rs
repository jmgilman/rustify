use std::error::Error as StdError;
use thiserror::Error;
use url::ParseError;

use crate::enums::RequestType;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Error sending HTTP request")]
    RequestError {
        source: Box<dyn StdError>,
        method: RequestType,
        url: String,
        body: Option<String>,
    },
    #[error("Error building HTTP request")]
    RequestBuildError {
        source: Box<dyn StdError>,
        method: RequestType,
        url: String,
    },
    #[error("Error retrieving HTTP response")]
    ResponseError { source: Box<dyn StdError> },
    #[error("Error parsing HTTP response")]
    ResponseParseError {
        source: Box<dyn StdError>,
        content: String,
    },
    #[error("Server returned error")]
    ServerResponseError {
        url: String,
        code: u16,
        content: String,
    },
    #[error("Error parsing URL")]
    UrlParseError { source: ParseError, url: String },
}
