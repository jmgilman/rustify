use std::error::Error as StdError;
use thiserror::Error;
use url::ParseError;

use crate::{client::Request, enums::RequestType};

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
        request: Request,
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
        content: Vec<u8>,
    },
    #[error("Server returned error")]
    ServerResponseError {
        url: String,
        code: u16,
        content: Vec<u8>,
    },
    #[error("Error parsing URL")]
    UrlParseError { source: ParseError, url: String },
}
