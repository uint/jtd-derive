use std::collections::HashMap;

use syn::{Field, Lit, Meta, MetaList, MetaNameValue, NestedMeta};

use super::{collect_attrs, ATTR_IDENT};
use crate::iter_ext::IterExt as _;

#[derive(Default)]
pub struct FieldCtx {
    pub metadata: HashMap<String, String>,
}

impl FieldCtx {
    pub fn from_input(input: &Field) -> Result<Self, syn::Error> {
        let mut field = Self::default();

        let params = collect_attrs(&input.attrs, ATTR_IDENT)?;
        params
            .map(|p| {
                match p
                    .path()
                    .get_ident()
                    .ok_or_else(|| {
                        syn::Error::new_spanned(p.path(), "jtd-derive parameter must be an ident")
                    })?
                    .to_string()
                    .as_str()
                {
                    "metadata" => {
                        if let Meta::List(MetaList { nested, .. }) = p {
                            let metadata = nested
                                .into_iter()
                                .map(|nested_meta| {
                                    if let NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                                        path,
                                        lit,
                                        ..
                                    })) = nested_meta
                                    {
                                        let key = path.get_ident().map(ToString::to_string).ok_or(
                                            syn::Error::new_spanned(
                                                path,
                                                "expected an ident, not a multi-segment path",
                                            ),
                                        )?;
                                        if let Lit::Str(val) = lit {
                                            Ok((key, val.value()))
                                        } else {
                                            Err(syn::Error::new_spanned(
                                                lit,
                                                "expected string literal",
                                            ))
                                        }
                                    } else {
                                        Err(syn::Error::new_spanned(
                                            nested_meta,
                                            "expected key-value pair",
                                        ))
                                    }
                                })
                                .collect_fallible()?;

                            field.metadata = metadata;
                            Ok(())
                        } else {
                            Err(syn::Error::new_spanned(
                                p,
                                "the `metadata` parameter must be a list of key-value pairs",
                            ))
                        }
                    }
                    _ => Err(syn::Error::new_spanned(
                        p.path(),
                        "unknown jtd-derive parameter",
                    )),
                }
            })
            .collect_fallible()?;

        Ok(field)
    }
}
