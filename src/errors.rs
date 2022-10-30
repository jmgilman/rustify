//! Contains the common error enum used across this crate
use thiserror::Error;

use crate::enums::RequestMethod;

/// The general error type returned by this crate
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Error parsing endpoint into data: {source}")]
    DataParseError { source: anyhow::Error },
    #[error("Error building endpoint request: {source}")]
    EndpointBuildError { source: anyhow::Error },
    #[error("An error occurred in processing the request: {source}")]
    GenericError { source: anyhow::Error },
    #[error("Error sending HTTP request: {source}")]
    RequestError {
        source: anyhow::Error,
        url: String,
        method: String,
    },
    #[error("Error building HTTP request: {source}")]
    RequestBuildError {
        source: http::Error,
        method: RequestMethod,
        url: String,
    },
    #[error("Error building request for Reqwest crate: {source}")]
    ReqwestBuildError { source: reqwest::Error },
    #[error("Error retrieving HTTP response: {source}")]
    ResponseError { source: anyhow::Error },
    #[error("Error parsing server response as UTF-8: {source}")]
    ResponseConversionError {
        source: anyhow::Error,
        content: Vec<u8>,
    },
    #[error("Error parsing HTTP response: {source}")]
    ResponseParseError {
        source: anyhow::Error,
        content: Option<String>,
    },
    #[error("Server returned error (HTTP {code})")]
    ServerResponseError { code: u16, content: Option<String> },
    #[error("Error building URL: {source}")]
    UrlBuildError { source: http::uri::InvalidUri },
    #[error("Error serializing URL query parameters: {source}")]
    UrlQueryParseError { source: anyhow::Error },
    #[error("Error parsing URL: {source}")]
    UrlParseError { source: url::ParseError },
}
