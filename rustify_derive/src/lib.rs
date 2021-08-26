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

use std::collections::{HashMap, HashSet};

use error::Error;
use params::Parameters;
use proc_macro2::Span;
use quote::quote;
use regex::Regex;
use syn::{self, Generics, Ident, Meta};

const MACRO_NAME: &str = "Endpoint";
const ATTR_NAME: &str = "endpoint";
const DATA_NAME: &str = "data";
const QUERY_NAME: &str = "query";

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
/// Searches the given map of (field name -> attribute parameter list) and looks
/// for any field which contains `QUERY_NAME`. A list of all field names which
/// are marked with the query parameter is generated and then mapped into a
/// list of token streams which appear as below:
/// ```
/// vec!(("field_name".to_string(), serde_json::value::to_value(&self.field_name).unwrap()))
/// ```
/// If no fields are marked the method is not generated which allows the trait
/// method to be used (defaults to empty vec).
fn gen_query(fields: &HashMap<Ident, HashSet<Meta>>) -> proc_macro2::TokenStream {
    // Collect all fields that have a query parameter attached
    let mut query_fields = Vec::<&Ident>::new();
    for (key, value) in fields.iter() {
        for attr in value.iter() {
            if attr.path().is_ident(QUERY_NAME) {
                query_fields.push(key);
            }
        }
    }

    match query_fields.is_empty() {
        false => {
            // Vec of ("#id".to_string(), serde_json::value::to_value(&self.#id).unwrap())
            let exprs = query_fields
                .iter()
                .map(|id| {
                    let id_str = id.to_string();
                    quote! {(#id_str.to_string(), serde_json::value::to_value(&self.#id).unwrap()) }
                })
                .collect::<Vec<proc_macro2::TokenStream>>();

            // Construct query function
            quote! {
                fn query(&self) -> Vec<(String, Value)> {
                    vec!(#(#exprs),*)
                }
            }
        }
        true => quote! {},
    }
}

fn gen_data(fields: &HashMap<Ident, HashSet<Meta>>) -> Result<proc_macro2::TokenStream, Error> {
    // Find data fields
    let mut data_fields = Vec::<&Ident>::new();
    for (key, value) in fields.iter() {
        for attr in value.iter() {
            if attr.path().is_ident(DATA_NAME) {
                data_fields.push(key);
            }
        }
    }

    // Return if empty
    if data_fields.is_empty() {
        return Ok(quote! {});
    }

    // Only allow a single data field
    if data_fields.len() > 1 {
        return Err(Error::new(
            data_fields[1].span(),
            "May only mark one field as the data field",
        ));
    }

    // Determine data type and return correct enum variant
    let id = data_fields[0];
    Ok(quote! {
        fn data(&self) -> Option<Bytes> {
            Some(self.#id.clone())
        }
    })
}

/// Generates `builder()` and `exec_*` helper methods for use with
/// `derive_builder`.
///
/// Adds an implementation to the base struct which provides a `builder` method
/// for returning instances of the Builder variant of the struct. This removes
/// the need to explicitely import it.
fn gen_builder(id: &Ident, generics: &Generics) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let builder_id: syn::Type =
        syn::parse_str(format!("{}Builder", id.to_string()).as_str()).unwrap();
    let builder_func: syn::Expr =
        syn::parse_str(format!("{}Builder::default()", id.to_string()).as_str()).unwrap();

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
    let attrs = match parse::attributes(&s.ast().attrs) {
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

    // Generate path string
    let path = match gen_path(&path) {
        Ok(a) => a,
        Err(e) => return e.into_tokens(),
    };

    // Generate query function
    let query = gen_query(&field_attrs);

    // Generate data
    let data = match gen_data(&field_attrs) {
        Ok(v) => v,
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
    let const_name = format!("_DERIVE_Endpoint_FOR_{}", id.to_string());
    let const_ident = Ident::new(const_name.as_str(), Span::call_site());
    quote! {
        const #const_ident: () = {
            use ::bytes::Bytes;
            use ::rustify::client::Client;
            use ::rustify::endpoint::Endpoint;
            use ::rustify::enums::{RequestMethod, RequestType, ResponseType};
            use ::rustify::errors::ClientError;
            use ::serde_json::Value;

            impl #impl_generics Endpoint for #id #ty_generics #where_clause {
                type Response = #response;
                const REQUEST_BODY_TYPE: RequestType = RequestType::#request_type;
                const RESPONSE_BODY_TYPE: ResponseType = ResponseType::#response_type;

                fn path(&self) -> String {
                    #path
                }

                fn method(&self) -> RequestMethod {
                    RequestMethod::#method
                }

                #query

                #data
            }

            #builder
        };
    }
}

synstructure::decl_derive!([Endpoint, attributes(endpoint)] => endpoint_derive);
