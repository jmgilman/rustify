//! Contains the common error enum used across this crate
use thiserror::Error;

use crate::enums::RequestMethod;

/// The general error type returned by this crate
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Error parsing endpoint into data")]
    DataParseError { source: anyhow::Error },
    #[error("Error building endpoint request")]
    EndpointBuildError { source: anyhow::Error },
    #[error("An error occurred in processing the request")]
    GenericError { source: anyhow::Error },
    #[error("Error sending HTTP request")]
    RequestError {
        source: anyhow::Error,
        url: String,
        method: String,
    },
    #[error("Error building HTTP request")]
    RequestBuildError {
        source: http::Error,
        method: RequestMethod,
        url: String,
    },

    #[cfg(feature = "reqwest")]
    #[error("Error building request for Reqwest crate")]
    ReqwestBuildError { source: reqwest::Error },

    #[error("Error retrieving HTTP response")]
    ResponseError { source: anyhow::Error },
    #[error("Error parsing server response as UTF-8")]
    ResponseConversionError {
        source: anyhow::Error,
        content: Vec<u8>,
    },
    #[error("Error parsing HTTP response")]
    ResponseParseError {
        source: anyhow::Error,
        content: Option<String>,
    },
    #[error("Server returned error")]
    ServerResponseError { code: u16, content: Option<String> },
    #[error("Error building URL")]
    UrlBuildError { source: http::uri::InvalidUri },
    #[error("Error serializing URL query parameters")]
    UrlQueryParseError { source: anyhow::Error },
    #[error("Error parsing URL")]
    UrlParseError { source: url::ParseError },
}
