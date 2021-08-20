//! Provides a derive macro for easily implementing an Endpoint from the
//! `rustify` crate. See the documentation for `rustify` for details on how
//! to use this macro.

#[macro_use]
extern crate synstructure;
extern crate proc_macro;

mod error;
mod params;
mod parse;

use error::Error;
use proc_macro2::Span;
use quote::quote;
use regex::Regex;
use syn::{self, Ident};

const MACRO_NAME: &str = "Endpoint";
const ATTR_NAME: &str = "endpoint";
const QUERY_NAME: &str = "query";

fn gen_action(path: &syn::LitStr) -> Result<proc_macro2::TokenStream, Error> {
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

fn endpoint_derive(s: synstructure::Structure) -> proc_macro2::TokenStream {
    // Parse `endpoint` attributes attached to input struct
    let attrs = match parse::attributes(&s.ast().attrs) {
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

    // Parse the attribute as a key/value pair list
    let kv = match parse::attr_kv(&attrs[0]) {
        Ok(v) => v,
        Err(e) => return e.into_tokens(),
    };

    // Create map from key/value pair list
    let map = match parse::to_map(&kv) {
        Ok(v) => v,
        Err(e) => return e.into_tokens(),
    };

    let params = match params::Parameters::new(map) {
        Ok(v) => v,
        Err(e) => return e.into_tokens(),
    };

    // Parse arguments
    let path = params.path;
    let method = params.method;
    let result = params.result;
    let request_type = params.request_type;
    let response_type = params.response_type;

    // Capture generic information
    let (impl_generics, ty_generics, where_clause) = s.ast().generics.split_for_impl();

    // Hacky variable substitution
    let action = match gen_action(&path) {
        Ok(a) => a,
        Err(e) => return e.into_tokens(),
    };

    // Gather any query parameters
    let mut query_params: Vec<proc_macro2::TokenStream> = Vec::new();
    if let syn::Data::Struct(data) = &s.ast().data {
        for field in data.fields.iter() {
            if field.ident.clone().unwrap() == QUERY_NAME {
                let id = &field.ident;
                let id_str = field.ident.as_ref().unwrap().to_string();
                let expr = quote! {(#id_str.to_string(), serde_json::value::to_value(&self.#id).unwrap()) };
                query_params.push(expr);
            } else {
                for attr in field.attrs.iter() {
                    if attr.path.is_ident(QUERY_NAME) {
                        let id = &field.ident;
                        let id_str = field.ident.as_ref().unwrap().to_string();
                        let expr = quote! {(#id_str.to_string(), serde_json::value::to_value(&self.#id).unwrap()) };
                        query_params.push(expr);
                    }
                }
            }
        }
    }

    let query = match query_params.is_empty() {
        false => quote! {
            fn query(&self) -> Vec<(String, Value)> {
                vec!(#(#query_params),*)
            }
        },
        true => quote! {},
    };

    // Gather data
    //gather_attributes(&s.ast().data);

    // Helper functions for the builder architecture
    let id = s.ast().ident.clone();
    let builder_id: syn::Type =
        syn::parse_str(format!("{}Builder", s.ast().ident.to_string()).as_str()).unwrap();
    let builder_func: syn::Expr =
        syn::parse_str(format!("{}Builder::default()", s.ast().ident.to_string()).as_str())
            .unwrap();
    let builder = match params.builder {
        true => quote! {
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
        },
        false => quote! {},
    };

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

synstructure::decl_derive!([Endpoint, attributes(endpoint, query)] => endpoint_derive);
