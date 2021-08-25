use std::error::Error as StdError;
use thiserror::Error;
use url::ParseError;

use crate::enums::RequestMethod;

/// The general error type returned by this crate
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Error parsing endpoint into data")]
    DataParseError { source: Box<dyn StdError> },
    #[error("Error building endpoint request")]
    EndpointBuildError { source: Box<dyn StdError> },
    #[error("An error occurred in processing the request")]
    GenericError { source: Box<dyn StdError> },
    #[error("Error sending HTTP request")]
    RequestError {
        source: Box<dyn StdError>,
        url: String,
        method: String,
    },
    #[error("Error building HTTP request")]
    RequestBuildError {
        source: Box<dyn StdError>,
        method: RequestMethod,
        url: String,
    },
    #[error("Error retrieving HTTP response")]
    ResponseError { source: Box<dyn StdError> },
    #[error("Error parsing server response as UTF-8")]
    ResponseConversionError {
        source: Box<dyn StdError>,
        content: Vec<u8>,
    },
    #[error("Error parsing HTTP response")]
    ResponseParseError {
        source: Box<dyn StdError>,
        content: Option<String>,
    },
    #[error("Server returned error")]
    ServerResponseError { code: u16, content: Option<String> },
    #[error("Error serializing URL query parameters")]
    UrlQueryParseError { source: Box<dyn StdError> },
    #[error("Error parsing URL")]
    UrlParseError { source: ParseError, url: String },
}
