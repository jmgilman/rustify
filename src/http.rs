use crate::{
    enums::{RequestMethod, RequestType, ResponseType},
    errors::ClientError,
};
use http::Request as HttpRequest;
use http::Uri;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use url::Url;

pub fn build_body<S: Serialize>(
    object: &S,
    ty: RequestType,
    data: Option<&[u8]>,
) -> Result<Vec<u8>, ClientError> {
    match data {
        Some(d) => Ok(d.to_vec()),
        None => match ty {
            RequestType::JSON => {
                let parse_data =
                    serde_json::to_string(object).map_err(|e| ClientError::DataParseError {
                        source: Box::new(e),
                    })?;
                Ok(match parse_data.as_str() {
                    "null" => "".to_string(),
                    "{}" => "".to_string(),
                    _ => parse_data,
                }
                .into_bytes())
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
    data: Vec<u8>,
) -> Result<HttpRequest<Vec<u8>>, ClientError> {
    let uri = build_url(base, path, query)?;

    let method_err = method.clone();
    let uri_err = uri.to_string();
    HttpRequest::builder()
        .uri(uri)
        .method(method)
        .body(data)
        .map_err(|e| ClientError::RequestBuildError {
            source: Box::new(e),
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

    let mut url = Url::parse(base).map_err(|e| ClientError::UrlParseError {
        url: base.to_string(),
        source: e,
    })?;
    url.path_segments_mut().unwrap().extend(path.split('/'));

    {
        let mut pairs = url.query_pairs_mut();
        let serializer = serde_urlencoded::Serializer::new(&mut pairs);
        query
            .serialize(serializer)
            .map_err(|e| ClientError::UrlQueryParseError {
                source: Box::new(e),
            })?;
    }

    Ok(url.to_string().parse::<Uri>().unwrap())
}

/// Parses a response body into the [Endpoint::Result], choosing a deserializer
/// based on [Endpoint::RESPONSE_BODY_TYPE].
pub fn parse<T: DeserializeOwned>(ty: ResponseType, body: &[u8]) -> Result<Option<T>, ClientError> {
    if body.is_empty() {
        return Ok(None);
    }

    match ty {
        ResponseType::JSON => {
            serde_json::from_slice(body).map_err(|e| ClientError::ResponseParseError {
                source: Box::new(e),
                content: String::from_utf8(body.to_vec()).ok(),
            })
        }
    }
}
