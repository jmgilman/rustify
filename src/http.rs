//! Contains helper functions for working with HTTP requests and responses.

use crate::{
    enums::{RequestMethod, RequestType, ResponseType},
    errors::ClientError,
};
use http::{Request, Uri};
use serde::{de::DeserializeOwned, Serialize};
use url::Url;

/// Builds a request body by serializing an object using a serializer determined
/// by the [RequestType].
pub fn build_body(object: &impl Serialize, ty: RequestType) -> Result<Vec<u8>, ClientError> {
    match ty {
        RequestType::JSON => {
            let parse_data = serde_json::to_string(object)
                .map_err(|e| ClientError::DataParseError { source: e.into() })?;
            Ok(match parse_data.as_str() {
                "null" => "".as_bytes().to_vec(),
                "{}" => "".as_bytes().to_vec(),
                _ => parse_data.as_bytes().to_vec(),
            })
        }
    }
}

/// Builds a query string by serializing an object
pub fn build_query(object: &impl Serialize) -> Result<String, ClientError> {
    serde_urlencoded::to_string(object)
        .map_err(|e| ClientError::UrlQueryParseError { source: e.into() })
}

/// Builds a [Request] using the given [Endpoint] and base URL
pub fn build_request(
    base: &str,
    path: &str,
    method: RequestMethod,
    query: Option<String>,
    data: Option<Vec<u8>>,
) -> Result<Request<Vec<u8>>, ClientError> {
    let uri = build_url(base, path, query)?;

    let method_err = method.clone();
    let uri_err = uri.to_string();
    Request::builder()
        .uri(uri)
        .method(method)
        .body(data.unwrap_or_else(|| Vec::<u8>::new()))
        .map_err(|e| ClientError::RequestBuildError {
            source: e,
            method: method_err,
            url: uri_err,
        })
}

/// Combines the given base URL with the relative URL path from this
/// Endpoint to create a fully qualified URL.
pub fn build_url(base: &str, path: &str, query: Option<String>) -> Result<Uri, ClientError> {
    log::info!(
        "Building endpoint url from {} base URL and {} action",
        base,
        path,
    );

    let mut url = Url::parse(base).map_err(|e| ClientError::UrlParseError { source: e })?;
    url.path_segments_mut().unwrap().extend(path.split('/'));
    if let Some(q) = query {
        url.set_query(Some(q.as_str()));
    }

    url.to_string()
        .parse::<Uri>()
        .map_err(|e| ClientError::UrlBuildError { source: e })
}

/// Parses a response body into the [Endpoint::Response], choosing a deserializer
/// based on [Endpoint::RESPONSE_BODY_TYPE].
pub fn parse<T: DeserializeOwned>(ty: ResponseType, body: &[u8]) -> Result<Option<T>, ClientError> {
    if body.is_empty() {
        return Ok(None);
    }

    match ty {
        ResponseType::JSON => {
            serde_json::from_slice(body).map_err(|e| ClientError::ResponseParseError {
                source: e.into(),
                content: String::from_utf8(body.to_vec()).ok(),
            })
        }
    }
}
