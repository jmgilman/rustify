//! Provides a derive macro for easily implementing an `Endpoint` from the
//! [rustify][1] crate. See the documentation for `rustify` for details on how
//! to use this macro.
//!
//! [1]: https://docs.rs/rustify/

#[macro_use]
extern crate synstructure;
extern crate proc_macro;

mod error;
mod params;
mod parse;

use std::{collections::HashMap, convert::TryFrom};

use error::Error;
use params::Parameters;
use proc_macro2::Span;
use quote::quote;
use regex::Regex;
use syn::{self, spanned::Spanned, Field, Generics, Ident, Meta};

const MACRO_NAME: &str = "Endpoint";
const ATTR_NAME: &str = "endpoint";

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum EndpointAttribute {
    Body,
    Query,
    Raw,
    Skip,
    Untagged,
}

impl TryFrom<&Meta> for EndpointAttribute {
    type Error = Error;
    fn try_from(m: &Meta) -> Result<Self, Self::Error> {
        match m.path().get_ident() {
            Some(i) => match i.to_string().to_lowercase().as_str() {
                "body" => Ok(EndpointAttribute::Body),
                "query" => Ok(EndpointAttribute::Query),
                "raw" => Ok(EndpointAttribute::Raw),
                "skip" => Ok(EndpointAttribute::Skip),
                _ => Err(Error::new(
                    m.span(),
                    format!("Unknown attribute: {}", i).as_str(),
                )),
            },
            None => Err(Error::new(m.span(), "Invalid attribute")),
        }
    }
}

/// Generates the path string for the endpoint.
///
/// The string supplied by the end-user supports basic interpolation using curly
/// braces. For example,
/// ```
/// endpoint(path = "user/{self.name}")
/// ```
/// Should produce:
/// ```
/// format!("user/{}", self.name);
/// ```
/// This is currently accomplished using a basic regular expression which
/// matches contents in the braces, extracts them out, leaving behind the empty
/// braces and placing the contents into the proper position in `format!`.
///
/// If no interpolation is needed the user provided string is fed into
/// `String::from` without modification.
fn gen_path(path: &syn::LitStr) -> Result<proc_macro2::TokenStream, Error> {
    let re = Regex::new(r"\{(.*?)\}").unwrap();
    let mut fmt_args: Vec<syn::Expr> = Vec::new();
    for cap in re.captures_iter(path.value().as_str()) {
        let expr = syn::parse_str(&cap[1]);
        match expr {
            Ok(ex) => fmt_args.push(ex),
            Err(_) => {
                return Err(Error::new(
                    path.span(),
                    format!("Failed parsing format argument as expression: {}", &cap[1]).as_str(),
                ));
            }
        }
    }
    let path = syn::LitStr::new(
        re.replace_all(path.value().as_str(), "{}")
            .to_string()
            .as_str(),
        Span::call_site(),
    );

    if !fmt_args.is_empty() {
        Ok(quote! {
            format!(#path, #(#fmt_args),*)
        })
    } else {
        Ok(quote! {
            String::from(#path)
        })
    }
}

/// Generates the query method for generating query parameters.
///
/// If any fields are found with the [EndpointAttribute::Query] attribute they
/// are combined into a new struct and then serialized into a query string. If
/// the attribute is not found on any of the fields the query method is not
/// generated.
fn gen_query(
    fields: &HashMap<EndpointAttribute, Vec<Field>>,
    serde_attrs: &[Meta],
) -> proc_macro2::TokenStream {
    let query_fields = fields.get(&EndpointAttribute::Query);
    if let Some(v) = query_fields {
        // Construct query function
        let temp = parse::fields_to_struct(v, serde_attrs);
        quote! {
            fn query(&self) -> Result<Option<String>, rustify::errors::ClientError> {
                #temp

                Ok(Some(rustify::http::build_query(&__temp)?))
            }
        }
    } else {
        quote! {}
    }
}

/// Generates the body method for generating the request body.
///
/// The final result is determined by which attributes are present and/or
/// missing on the struct fields. The following order is respected:
///
/// * If a field is found with the [EndpointAttribute::Raw] attribute that field
///   is returned directly as the request body. The assumption is this field
///   will always be a [Vec<u8>].
/// * If any fields are found with the [EndpointAttribute::Body] attribute they
///   are combined into a new struct and then serialized into the request body
///   depending on the request type of the Endpoint.
/// * If neither of the above two conditions are true, and there are fields
///   found that don't have any attribute, those fields are combined into a new
///   struct and then serialized into the request body depending on the request
///   type of the Endpoint.
/// * If none of the above is true, the body method is not generated.
fn gen_body(
    fields: &HashMap<EndpointAttribute, Vec<Field>>,
    serde_attrs: &[Meta],
) -> Result<proc_macro2::TokenStream, Error> {
    // Check for a raw field first
    if let Some(v) = fields.get(&EndpointAttribute::Raw) {
        if v.len() > 1 {
            return Err(Error::new(v[1].span(), "May only mark one field as raw"));
        }

        let id = v[0].ident.clone().unwrap();
        Ok(quote! {
            fn body(&self) -> Result<Option<Vec<u8>>, rustify::errors::ClientError>{
                Ok(Some(self.#id.clone()))
            }
        })
    // Then for any body fields
    } else if let Some(v) = fields.get(&EndpointAttribute::Body) {
        let temp = parse::fields_to_struct(v, serde_attrs);
        Ok(quote! {
            fn body(&self) -> Result<Option<Vec<u8>>, rustify::errors::ClientError> {
                #temp

                Ok(Some(rustify::http::build_body(&__temp, Self::REQUEST_BODY_TYPE)?))
            }
        })
    // Then for any untagged fields
    } else if let Some(v) = fields.get(&EndpointAttribute::Untagged) {
        let temp = parse::fields_to_struct(v, serde_attrs);
        Ok(quote! {
            fn body(&self) -> Result<Option<Vec<u8>>, rustify::errors::ClientError> {
                #temp

                Ok(Some(rustify::http::build_body(&__temp, Self::REQUEST_BODY_TYPE)?))
            }
        })
    // Leave it undefined if no body fields found
    } else {
        Ok(quote! {})
    }
}

/// Generates `builder()` and `exec_*` helper methods for use with
/// `derive_builder`.
///
/// Adds an implementation to the base struct which provides a `builder` method
/// for returning instances of the Builder variant of the struct. This removes
/// the need to explicitly import it.
fn gen_builder(id: &Ident, generics: &Generics) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let builder_id: syn::Type = syn::parse_str(format!("{}Builder", id).as_str()).unwrap();
    let builder_func: syn::Expr =
        syn::parse_str(format!("{}Builder::default()", id).as_str()).unwrap();

    quote! {
        impl #impl_generics #id #ty_generics #where_clause {
            pub fn builder() -> #builder_id #ty_generics {
                #builder_func
            }
        }
    }
}

/// Parses parameters passed into the `endpoint` attribute attached to the
/// struct.
fn parse_params(attr: &Meta) -> Result<Parameters, Error> {
    // Parse the attribute as a key/value pair list
    let kv = parse::attr_kv(attr)?;

    // Create map from key/value pair list
    let map = parse::to_map(&kv)?;

    // Convert map to Parameters
    params::Parameters::new(map)
}

/// Implements `Endpoint` on the provided struct.
fn endpoint_derive(s: synstructure::Structure) -> proc_macro2::TokenStream {
    // Parse `endpoint` attributes attached to input struct
    let attrs = match parse::attributes(&s.ast().attrs, ATTR_NAME) {
        Ok(v) => v,
        Err(e) => return e.into_tokens(),
    };

    // Parse `endpoint` attributes attached to input struct fields
    let field_attrs = match parse::field_attributes(&s.ast().data) {
        Ok(v) => v,
        Err(e) => return e.into_tokens(),
    };

    // Verify attribute is present
    if attrs.is_empty() {
        return Error::new(
            Span::call_site(),
            format!(
                "Deriving `{}` requires attaching an `{}` attribute",
                MACRO_NAME, ATTR_NAME
            )
            .as_str(),
        )
        .into_tokens();
    }

    // Verify there's only one instance of the attribute present
    if attrs.len() > 1 {
        return Error::new(
            Span::call_site(),
            format!("Cannot define the {} attribute more than once", ATTR_NAME).as_str(),
        )
        .into_tokens();
    }

    // Parse endpoint attribute parameters
    let params = match parse_params(&attrs[0]) {
        Ok(v) => v,
        Err(e) => return e.into_tokens(),
    };

    let path = params.path;
    let method = params.method;
    let response = params.response;
    let request_type = params.request_type;
    let response_type = params.response_type;
    let id = &s.ast().ident;

    // Find serde attributes
    let serde_attrs = parse::attributes(&s.ast().attrs, "serde");
    let serde_attrs = serde_attrs.unwrap_or_default();

    // Generate path string
    let path = match gen_path(&path) {
        Ok(a) => a,
        Err(e) => return e.into_tokens(),
    };

    // Generate query function
    let query = gen_query(&field_attrs, &serde_attrs);

    // Generate body function
    let body = match gen_body(&field_attrs, &serde_attrs) {
        Ok(d) => d,
        Err(e) => return e.into_tokens(),
    };

    // Generate helper functions when deriving Builder
    let builder = match params.builder {
        true => gen_builder(&s.ast().ident, &s.ast().generics),
        false => quote! {},
    };

    // Capture generic information
    let (impl_generics, ty_generics, where_clause) = s.ast().generics.split_for_impl();

    // Generate Endpoint implementation
    quote! {
        impl #impl_generics rustify::endpoint::Endpoint for #id #ty_generics #where_clause {
            type Response = #response;
            const REQUEST_BODY_TYPE: rustify::enums::RequestType = rustify::enums::RequestType::#request_type;
            const RESPONSE_BODY_TYPE: rustify::enums::ResponseType = rustify::enums::ResponseType::#response_type;

            fn path(&self) -> String {
                #path
            }

            fn method(&self) -> rustify::enums::RequestMethod {
                rustify::enums::RequestMethod::#method
            }

            #query


            #body
        }

        #builder
    }
}

synstructure::decl_derive!([Endpoint, attributes(endpoint)] => endpoint_derive);
