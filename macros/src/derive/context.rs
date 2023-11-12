mod container;
mod field;

pub use container::Container;
pub use field::FieldCtx;

use sdi::attr::RenameRule;
use serde_derive_internals as sdi;
use syn::{Attribute, Lit, Meta, MetaNameValue, NestedMeta};

use crate::iter_ext::IterExt as _;

const ATTR_IDENT: &str = "typedef";
const SERDE_ATTR_IDENT: &str = "serde";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagType {
    External,
    Internal(String),
}

impl Default for TagType {
    fn default() -> Self {
        Self::External
    }
}

fn collect_attrs(
    attrs: &[Attribute],
    path: &str,
) -> Result<impl Iterator<Item = Meta>, syn::Error> {
    let attrs = attrs.iter().filter(|attr| {
        attr.path
            .get_ident()
            .map(|ident| *ident == path)
            .unwrap_or(false)
    });

    let parse_attr = |attr: &Attribute| -> Result<Vec<Meta>, syn::Error> {
        let meta = attr.parse_meta()?;
        if let Meta::List(l) = meta {
            let inner_metas = l
                .nested
                .into_iter()
                .map(|n| {
                    if let NestedMeta::Meta(m) = n {
                        Ok(m)
                    } else {
                        Err(syn::Error::new_spanned(n, "literals are not allowed here"))
                    }
                })
                .collect_fallible()?;

            Ok(inner_metas)
        } else {
            Err(syn::Error::new_spanned(
                meta,
                format!(
                    "{0} attributes are expected to take this form: #[{0}(...)]",
                    path
                ),
            ))
        }
    };

    Ok(attrs
        .map(parse_attr)
        .collect_fallible::<Vec<_>>()?
        .into_iter()
        .flatten())
}

fn parse_rename_rule(args: impl Iterator<Item = Meta>) -> Option<RenameRule> {
    let rename_all_args = args.filter(|meta| {
        meta.path()
            .get_ident()
            .map(|id| id.to_string().as_str() == "rename_all")
            .unwrap_or_default()
    });

    rename_all_args
        .filter_map(|meta| -> Option<RenameRule> {
            match meta {
                Meta::Path(_) => None,
                Meta::List(l) => l
                    .nested
                    .iter()
                    .filter_map(|nested| {
                        if let NestedMeta::Meta(Meta::NameValue(name_value)) = nested {
                            if !name_value
                                .path
                                .get_ident()
                                .map(|id| id.to_string().as_str() == "deserialize")
                                .unwrap_or_default()
                            {
                                return None;
                            }

                            if let Lit::Str(s) = &name_value.lit {
                                RenameRule::from_str(&s.value()).ok()
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .last(),
                Meta::NameValue(MetaNameValue { lit, .. }) => {
                    if let Lit::Str(s) = lit {
                        RenameRule::from_str(&s.value()).ok()
                    } else {
                        None
                    }
                }
            }
        })
        .last()
}
