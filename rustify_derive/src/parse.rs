use std::collections::HashMap;

use crate::Error;
use syn::{spanned::Spanned, Attribute, Ident, LitStr, Meta, MetaNameValue, NestedMeta};

pub fn attr_list(attr: &Meta) -> Result<Vec<Meta>, Error> {
    let mut result = Vec::<Meta>::new();
    if let Meta::List(list) = &attr {
        if list.nested.is_empty() {
            return Err(Error::new(attr.span(), "Attribute cannot be empty"));
        }

        for nested in list.nested.iter() {
            if let NestedMeta::Meta(nested_meta) = nested {
                result.push(nested_meta.clone())
            } else {
                return Err(Error::new(
                    nested.span(),
                    "Attribute cannot contain any literals",
                ));
            }
        }

        Ok(result)
    } else {
        Err(Error::new(attr.span(), "Cannot parse attribute as list"))
    }
}

pub fn attr_kv(attr: &Meta) -> Result<Vec<MetaNameValue>, Error> {
    let meta_list = attr_list(attr)?;
    let mut result = Vec::<MetaNameValue>::new();
    for meta in meta_list.iter() {
        if let syn::Meta::NameValue(nv_meta) = meta {
            result.push(nv_meta.clone());
        } else {
            return Err(Error::new(
                attr.span(),
                "Cannot parse attribute as a key/value list",
            ));
        }
    }
    Ok(result)
}

pub fn to_map(values: &[MetaNameValue]) -> Result<HashMap<Ident, LitStr>, Error> {
    let mut map = HashMap::<Ident, LitStr>::new();
    for value in values.iter() {
        let id = value.path.get_ident().unwrap().clone();
        if let syn::Lit::Str(lit) = &value.lit {
            map.insert(id, lit.clone());
        } else {
            return Err(Error::new(
                value.span(),
                "Values must be in string literal form",
            ));
        }
    }

    Ok(map)
}

pub fn attributes(attrs: &[Attribute]) -> Result<Vec<Meta>, Error> {
    let mut result = Vec::<Meta>::new();
    for attr in attrs.iter() {
        let meta = attr.parse_meta().map_err(Error::from)?;
        match meta.path().is_ident(crate::ATTR_NAME) {
            true => {
                result.push(meta);
            }
            false => {}
        }
    }

    Ok(result)
}

pub fn field_attributes(data: &syn::Data) -> Result<Vec<Meta>, Error> {
    let mut result = Vec::<Meta>::new();
    if let syn::Data::Struct(data) = data {
        for field in data.fields.iter() {
            result.extend(attributes(&field.attrs)?)
        }
    }

    Ok(result)
}
