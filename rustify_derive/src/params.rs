use std::collections::HashMap;

use crate::Error;
use proc_macro2::Span;
use syn::{Expr, Ident, LitStr, Type};

#[derive(Default, Debug)]
pub struct ParametersBuilder {
    pub path: Option<LitStr>,
    pub method: Option<Expr>,
    pub result: Option<Type>,
    pub request_type: Option<Expr>,
    pub response_type: Option<Expr>,
    pub builder: Option<bool>,
}

#[derive(Debug)]
pub struct Parameters {
    pub path: LitStr,
    pub method: Expr,
    pub result: Type,
    pub request_type: Expr,
    pub response_type: Expr,
    pub builder: bool,
}

impl Parameters {
    pub fn new(map: HashMap<Ident, LitStr>) -> Result<Parameters, Error> {
        let mut builder = ParametersBuilder::default();
        for key in map.keys() {
            match key.to_string().as_str() {
                "path" => builder.path = Some(map[key].clone()),
                "method" => {
                    builder.method = Some(parse(&map[key])?);
                }
                "result" => {
                    builder.result = Some(parse(&map[key])?);
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
            result: builder
                .result
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

fn parse<T: syn::parse::Parse>(value: &LitStr) -> Result<T, Error> {
    value
        .parse()
        .map_err(|_| Error::new(value.span(), "Unable to parse value"))
}
