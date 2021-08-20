//! Provides a derive macro for easily implementing an Endpoint from the
//! `rustify` crate. See the documentation for `rustify` for details on how
//! to use this macro.

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
use syn::{self, Generics, Ident, Meta, Type};

const MACRO_NAME: &str = "Endpoint";
const ATTR_NAME: &str = "endpoint";
const QUERY_NAME: &str = "query";

fn action(path: &syn::LitStr) -> Result<proc_macro2::TokenStream, Error> {
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

fn query(fields: &HashMap<Ident, HashSet<Meta>>) -> proc_macro2::TokenStream {
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

fn builder(id: &Ident, result: &Type, generics: &Generics) -> proc_macro2::TokenStream {
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

        impl #impl_generics #builder_id #ty_generics #where_clause {
            pub fn exec<C: Client>(
                &self,
                client: &C,
            ) -> Result<Option<#result>, ClientError> {
                self.build().map_err(|e| { ClientError::EndpointBuildError { source: Box::new(e)}})?.exec(client)
            }

            pub fn exec_mut<C: Client, M: MiddleWare>(
                &self,
                client: &C,
                middle: &M,
            ) -> Result<Option<#result>, ClientError> {
                self.build().map_err(|e| { ClientError::EndpointBuildError { source: Box::new(e)}})?.exec_mut(client, middle)
            }
        }
    }
}

fn parse_params(attr: &Meta) -> Result<Parameters, Error> {
    // Parse the attribute as a key/value pair list
    let kv = parse::attr_kv(attr)?;

    // Create map from key/value pair list
    let map = parse::to_map(&kv)?;

    // Convert map to Parameters
    params::Parameters::new(map)
}

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
    let result = params.result;
    let request_type = params.request_type;
    let response_type = params.response_type;
    let id = &s.ast().ident;

    // Generate action string
    let action = match action(&path) {
        Ok(a) => a,
        Err(e) => return e.into_tokens(),
    };

    // Generate query function
    let query = query(&field_attrs);

    // Gather data
    //gather_attributes(&s.ast().data);

    // Generate helper functions when deriving Builder
    let builder = match params.builder {
        true => builder(&s.ast().ident, &result, &s.ast().generics),
        false => quote! {},
    };

    // Capture generic information
    let (impl_generics, ty_generics, where_clause) = s.ast().generics.split_for_impl();

    // Generate Endpoint implementation
    let const_name = format!("_DERIVE_Endpoint_FOR_{}", id.to_string());
    let const_ident = Ident::new(const_name.as_str(), Span::call_site());
    quote! {
        const #const_ident: () = {
            use ::rustify::client::Client;
            use ::rustify::endpoint::{Endpoint, MiddleWare};
            use ::rustify::enums::{RequestMethod, RequestType, ResponseType};
            use ::rustify::errors::ClientError;
            use ::serde_json::Value;

            impl #impl_generics Endpoint for #id #ty_generics #where_clause {
                type Result = #result;
                const REQUEST_BODY_TYPE: RequestType = RequestType::#request_type;
                const RESPONSE_BODY_TYPE: ResponseType = ResponseType::#response_type;

                fn action(&self) -> String {
                    #action
                }

                fn method(&self) -> RequestMethod {
                    RequestMethod::#method
                }

                #query
            }

            #builder
        };
    }
}

synstructure::decl_derive!([Endpoint, attributes(endpoint)] => endpoint_derive);
