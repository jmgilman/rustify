use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};

use crate::{EndpointAttribute, Error};
use proc_macro2::Span;
use syn::{
    spanned::Spanned, Attribute, Field, Ident, LitStr, Meta, MetaNameValue, NestedMeta, Type,
};

/// Returns all [Meta] values contained in a [Meta::List].
///
/// For example:
/// ```
/// #[endpoint(query, data)]
/// ```
/// Would return individual [Meta] values for `query` and `data`. This function
/// fails if the [Meta::List] is empty or contains any literals.
pub(crate) fn attr_list(attr: &Meta) -> Result<Vec<Meta>, Error> {
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

/// Returns all [MetaNameValue] values contained in a [Meta::List].
///
/// For example:
/// ```
/// #[endpoint(path = "my/path", method = "POST")]
/// ```
/// Would return individual [MetaNameValue] values for `path` and `method`. This
/// function fails if the [Meta::List] is empty, contains literals, or cannot
/// be parsed as name/value pairs.
pub(crate) fn attr_kv(attr: &Meta) -> Result<Vec<MetaNameValue>, Error> {
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

/// Converts a list of [MetaNameValue] values into a [HashMap].
///
/// For example, assuming the below has been parsed into [MetaNameValue]'s:
/// ```
/// #[endpoint(path = "my/path", method = "POST")]
/// ```
/// Would return a [HashMap] mapping individual ID's (i.e. `path` and `method`)
/// to their [LitStr] values (i.e. "m/path" and "POST"). This function fails if
/// the values cannot be parsed as string literals.
pub(crate) fn to_map(values: &[MetaNameValue]) -> Result<HashMap<Ident, LitStr>, Error> {
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

/// Searches a list of [Attribute]'s and returns any matching [crate::ATTR_NAME].
pub(crate) fn attributes(attrs: &[Attribute]) -> Result<Vec<Meta>, Error> {
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

/// Returns a mapping of endpoint attributes to a list of their fields.
///
/// Parses all [Attribute]'s on the given [syn::Field]'s, searching for any
/// attributes which match [crate::ATTR_NAME] and creating a map of attributes
/// to a list of their associated fields.
pub(crate) fn field_attributes(
    data: &syn::Data,
) -> Result<HashMap<EndpointAttribute, Vec<Field>>, Error> {
    let mut result = HashMap::<EndpointAttribute, Vec<Field>>::new();
    if let syn::Data::Struct(data) = data {
        for field in data.fields.iter() {
            // Collect all `endpoint` attributes attached to this field
            let attrs = attributes(&field.attrs)?;

            // Combine all meta parameters from each attribute
            let attrs = attrs
                .iter()
                .map(|a| attr_list(a))
                .collect::<Result<Vec<Vec<Meta>>, Error>>()?;

            // Flatten and eliminate duplicates
            let attrs = attrs.into_iter().flatten().collect::<HashSet<Meta>>();

            // Add this field to the list of fields for each attribute
            for attr in attrs.iter() {
                let attr_ty = EndpointAttribute::try_from(attr)?;
                match result.get_mut(&attr_ty) {
                    Some(r) => {
                        r.push(field.clone());
                    }
                    None => {
                        result.insert(attr_ty, vec![field.clone()]);
                    }
                }
            }
        }
    }

    Ok(result)
}
