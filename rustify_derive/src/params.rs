use std::collections::HashMap;

use crate::Error;
use proc_macro2::Span;
use syn::{Expr, Ident, LitStr, Type};

/// Used for building the parameter list for the derive function
#[derive(Default, Debug)]
pub struct ParametersBuilder {
    pub path: Option<LitStr>,
    pub method: Option<Expr>,
    pub response: Option<Type>,
    pub request_type: Option<Expr>,
    pub response_type: Option<Expr>,
    pub builder: Option<bool>,
}

/// Represents all valid parameters that can be passed to the derive function
#[derive(Debug)]
pub struct Parameters {
    pub path: LitStr,
    pub method: Expr,
    pub response: Type,
    pub request_type: Expr,
    pub response_type: Expr,
    pub builder: bool,
}

impl Parameters {
    /// Given a map of identities to literal strings, builds a new instance of
    /// [Parameters] using the contents of the map.
    ///
    /// The only required parameter is `path` and not providing it will cause
    /// the function to fail. All other parameters are optional and will have
    /// sane defaults provided if they are not found in the map.
    pub fn new(map: HashMap<Ident, LitStr>) -> Result<Parameters, Error> {
        let mut builder = ParametersBuilder::default();
        for key in map.keys() {
            match key.to_string().as_str() {
                "path" => builder.path = Some(map[key].clone()),
                "method" => {
                    builder.method = Some(parse(&map[key])?);
                }
                "response" => {
                    builder.response = Some(parse(&map[key])?);
                }
                "request_type" => {
                    builder.request_type = Some(parse(&map[key])?);
                }
                "response_type" => {
                    builder.response_type = Some(parse(&map[key])?);
                }
                "builder" => {
                    builder.builder = Some(true);
                }
                _ => {
                    return Err(Error::new(key.span(), "Unknown parameter"));
                }
            }
        }

        let params = Parameters {
            path: match builder.path {
                Some(p) => p,
                None => {
                    return Err(Error::new(
                        Span::call_site(),
                        "Missing required parameter: path",
                    ))
                }
            },
            method: builder
                .method
                .unwrap_or_else(|| syn::parse_str("GET").unwrap()),
            response: builder
                .response
                .unwrap_or_else(|| syn::parse_str("()").unwrap()),
            request_type: builder
                .request_type
                .unwrap_or_else(|| syn::parse_str("JSON").unwrap()),
            response_type: builder
                .response_type
                .unwrap_or_else(|| syn::parse_str("JSON").unwrap()),
            builder: builder.builder.unwrap_or(false),
        };

        Ok(params)
    }
}

/// Parses a [LitStr] into `T` and returns an error if it fails
fn parse<T: syn::parse::Parse>(value: &LitStr) -> Result<T, Error> {
    value
        .parse()
        .map_err(|_| Error::new(value.span(), "Unable to parse value"))
}
