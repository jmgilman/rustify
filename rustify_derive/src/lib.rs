//! Provides a derive macro for easily implementing an Endpoint from the
//! `rustify` crate. See the documentation for `rustify` for details on how
//! to use this macro.

#[macro_use]
extern crate synstructure;
extern crate proc_macro;

use proc_macro2::Span;
use quote::quote;
use regex::Regex;
use std::ops::Deref;
use syn::{self, spanned::Spanned, Ident};

const MACRO_NAME: &str = "Endpoint";
const ATTR_NAME: &str = "endpoint";
const QUERY_NAME: &str = "query";

#[derive(Debug)]
struct Error(proc_macro2::TokenStream);

impl Error {
    fn new(span: Span, message: &str) -> Error {
        Error(quote_spanned! { span =>
            compile_error!(#message);
        })
    }

    fn into_tokens(self) -> proc_macro2::TokenStream {
        self.0
    }
}

impl From<syn::Error> for Error {
    fn from(e: syn::Error) -> Error {
        Error(e.to_compile_error())
    }
}

#[derive(Default, Debug)]
struct Parameters {
    path: Option<syn::LitStr>,
    method: Option<syn::Expr>,
    result: Option<syn::Type>,
    transform: Option<syn::Expr>,
    builder: Option<bool>,
}

fn parse_attr(meta: &syn::Meta) -> Result<Parameters, Error> {
    let mut params = Parameters::default();
    if let syn::Meta::List(l) = meta {
        // Verify the attribute list isn't empty
        if l.nested.is_empty() {
            return Err(Error::new(
                meta.span(),
                format!(
                    "The `{}` attribute must be a list of name/value pairs",
                    ATTR_NAME
                )
                .as_str(),
            ));
        }

        // Collect name/value arguments
        let mut args: Vec<&syn::MetaNameValue> = Vec::new();
        for nm in l.nested.iter() {
            if let syn::NestedMeta::Meta(m) = nm {
                if let syn::Meta::NameValue(nv) = m {
                    args.push(nv);
                } else {
                    return Err(Error::new(
                        m.span(),
                        format!(
                            "The `{}` attribute must only contain name/value pairs",
                            ATTR_NAME
                        )
                        .as_str(),
                    ));
                }
            } else {
                return Err(Error::new(
                    nm.span(),
                    "The `action` attribute must not contain any literals",
                ));
            }
        }

        // Extract arguments
        for arg in args {
            if let syn::Lit::Str(val) = &arg.lit {
                match arg.path.get_ident().unwrap().to_string().as_str() {
                    "path" => {
                        params.path = Some(val.deref().clone());
                    }
                    "method" => {
                        params.method = Some(val.deref().clone().parse().map_err(|_| {
                            Error::new(arg.lit.span(), "Unable to parse value into expression")
                        })?);
                    }
                    "result" => {
                        params.result = Some(val.deref().clone().parse().map_err(|_| {
                            Error::new(arg.lit.span(), "Unable to parse value into expression")
                        })?);
                    }
                    "transform" => {
                        params.transform = Some(val.deref().clone().parse().map_err(|_| {
                            Error::new(arg.lit.span(), "Unable to parse value into expression")
                        })?);
                    }
                    "builder" => params.builder = Some(true),
                    _ => {
                        return Err(Error::new(arg.span(), "Unsupported argument"));
                    }
                }
            } else {
                return Err(Error::new(arg.span(), "Invalid value for argument"));
            }
        }
    } else {
        return Err(Error::new(
            meta.span(),
            format!(
                "The `{}` attribute must be a list of key/value pairs",
                ATTR_NAME
            )
            .as_str(),
        ));
    }
    Ok(params)
}

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
    let mut found_attr = false;
    let mut params = Parameters::default();
    for attr in &s.ast().attrs {
        match attr.parse_meta() {
            Ok(meta) => {
                if meta.path().is_ident(ATTR_NAME) {
                    found_attr = true;
                    match parse_attr(&meta) {
                        Ok(p) => {
                            params = p;
                        }
                        Err(e) => return e.into_tokens(),
                    }
                }
            }
            Err(e) => return e.to_compile_error(),
        }
    }

    if !found_attr {
        return Error::new(
            Span::call_site(),
            format!(
                "Must supply the `{}` attribute when deriving `{}`",
                ATTR_NAME, MACRO_NAME
            )
            .as_str(),
        )
        .into_tokens();
    }

    // Parse arguments
    let path = match params.path {
        Some(p) => p,
        None => {
            return Error::new(Span::call_site(), "Missing required `path` argument").into_tokens()
        }
    };
    let method = match params.method {
        Some(m) => m,
        None => syn::parse_str("GET").unwrap(),
    };
    let result = match params.result {
        Some(r) => r,
        None => syn::parse_str("EmptyEndpointResult").unwrap(),
    };

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

    // Optional post transformation method
    let transform = match params.transform {
        Some(t) => quote! {
            fn transform(&self, res: String) -> Result<String, ClientError> {
                #t(res)
            }
        },
        None => quote! {},
    };

    // Helper functions for the builder architecture
    let id = s.ast().ident.clone();
    let builder_id: syn::Type =
        syn::parse_str(format!("{}Builder", s.ast().ident.to_string()).as_str()).unwrap();
    let builder_func: syn::Expr =
        syn::parse_str(format!("{}Builder::default()", s.ast().ident.to_string()).as_str())
            .unwrap();
    let builder = match params.builder {
        Some(_) => quote! {
            impl #impl_generics #id #ty_generics #where_clause {
                pub fn builder() -> #builder_id #ty_generics {
                    #builder_func
                }
            }

            impl #impl_generics #builder_id #ty_generics #where_clause {
                pub fn execute<C: Client>(
                    &self,
                    client: &C,
                ) -> Result<Option<#result>, ClientError> {
                    self.build().map_err(|e| { ClientError::EndpointBuildError { source: Box::new(e)}})?.execute(client)
                }
            }
        },
        _ => quote! {},
    };

    // Generate Endpoint implementation
    let const_name = format!("_DERIVE_Endpoint_FOR_{}", id.to_string());
    let const_ident = Ident::new(const_name.as_str(), Span::call_site());
    quote! {
        const #const_ident: () = {
            use ::rustify::client::Client;
            use ::rustify::endpoint::{Endpoint, EmptyEndpointResult};
            use ::rustify::enums::RequestType;
            use ::rustify::errors::ClientError;
            use ::serde_json::Value;

            impl #impl_generics Endpoint for #id #ty_generics #where_clause {
                type Result = #result;

                fn action(&self) -> String {
                    #action
                }

                fn method(&self) -> RequestType {
                    RequestType::#method
                }

                #query

                #transform
            }

            #builder
        };
    }
}

synstructure::decl_derive!([Endpoint, attributes(endpoint, query)] => endpoint_derive);
