//! Contains helper functions for working with HTTP requests and responses.

use crate::{
    enums::{RequestMethod, RequestType},
    errors::ClientError,
};
use http::{Request, Uri};
use serde::Serialize;
use url::Url;

/// Builds a request body by serializing an object using a serializer determined
/// by the [RequestType].
#[instrument(skip(object), err)]
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

/// Builds a query string by serializing an object.
#[instrument(skip(object), err)]
pub fn build_query(object: &impl Serialize) -> Result<String, ClientError> {
    serde_urlencoded::to_string(object)
        .map_err(|e| ClientError::UrlQueryParseError { source: e.into() })
}

/// Builds a [Request] using the given [Endpoint][crate::Endpoint] and base URL.
#[instrument(skip(query, data), err)]
pub fn build_request(
    base: &str,
    path: &str,
    method: RequestMethod,
    query: Option<String>,
    data: Option<Vec<u8>>,
) -> Result<Request<Vec<u8>>, ClientError> {
    debug!("Building endpoint request");
    let uri = build_url(base, path, query)?;

    let uri_err = uri.to_string();
    Request::builder()
        .uri(uri)
        .method(method)
        .body(data.unwrap_or_default())
        .map_err(|e| ClientError::RequestBuildError {
            source: e,
            method,
            url: uri_err,
        })
}

/// Combines the given base URL, relative path, and optional query parameters
/// into a single [Uri].
#[instrument(skip(query), err)]
pub fn build_url(base: &str, path: &str, query: Option<String>) -> Result<Uri, ClientError> {
    let mut url = Url::parse(base).map_err(|e| ClientError::UrlParseError { source: e })?;
    url.path_segments_mut().unwrap().extend(path.split('/'));
    if let Some(q) = query {
        url.set_query(Some(q.as_str()));
    }

    url.to_string()
        .parse::<Uri>()
        .map_err(|e| ClientError::UrlBuildError { source: e })
}
