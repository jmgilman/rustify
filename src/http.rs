//! Contains helper functions for working with HTTP requests and responses.

use crate::{
    enums::{RequestMethod, RequestType, ResponseType},
    errors::ClientError,
};
use bytes::Bytes;
use http::{Request, Uri};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use url::Url;

/// Builds the body of a HTTP request in byte form using the given input.
///
/// If `data` is not None, the contents of data will be returned. Otherwise,
/// the `object` will be attempted to be serialized to a byte array using the
/// given [RequestType].
pub fn build_body(
    object: &impl Serialize,
    ty: RequestType,
    data: Option<Bytes>,
) -> Result<Bytes, ClientError> {
    match data {
        Some(d) => Ok(d),
        None => match ty {
            RequestType::JSON => {
                let parse_data = serde_json::to_string(object)
                    .map_err(|e| ClientError::DataParseError { source: e.into() })?;
                Ok(Bytes::from(match parse_data.as_str() {
                    "null" => "".to_string(),
                    "{}" => "".to_string(),
                    _ => parse_data,
                }))
            }
        },
    }
}

/// Builds a [Request] using the given [Endpoint] and base URL
pub fn build_request(
    base: &str,
    path: &str,
    method: RequestMethod,
    query: Vec<(String, Value)>,
    data: Bytes,
) -> Result<Request<Bytes>, ClientError> {
    let uri = build_url(base, path, query)?;

    let method_err = method.clone();
    let uri_err = uri.to_string();
    Request::builder()
        .uri(uri)
        .method(method)
        .body(data)
        .map_err(|e| ClientError::RequestBuildError {
            source: e,
            method: method_err,
            url: uri_err,
        })
}

/// Combines the given base URL with the relative URL path from this
/// Endpoint to create a fully qualified URL.
pub fn build_url(base: &str, path: &str, query: Vec<(String, Value)>) -> Result<Uri, ClientError> {
    log::info!(
        "Building endpoint url from {} base URL and {} action",
        base,
        path,
    );

    let mut url = Url::parse(base).map_err(|e| ClientError::UrlParseError { source: e })?;
    url.path_segments_mut().unwrap().extend(path.split('/'));

    {
        let mut pairs = url.query_pairs_mut();
        let serializer = serde_urlencoded::Serializer::new(&mut pairs);
        query
            .serialize(serializer)
            .map_err(|e| ClientError::UrlQueryParseError { source: e.into() })?;
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
