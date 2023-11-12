use std::collections::HashMap;

use sdi::attr::RenameRule;
use serde_derive_internals as sdi;
use syn::{DeriveInput, Lit, Meta, MetaList, MetaNameValue, NestedMeta, Type};

use super::{collect_attrs, TagType, ATTR_IDENT, SERDE_ATTR_IDENT};
use crate::iter_ext::IterExt as _;

#[derive(Default)]
pub struct Container {
    pub no_serde: bool,
    pub tag_type: TagType,
    pub deny_unknown_fields: bool,
    pub transparent: bool,
    pub type_from: Option<Type>,
    pub type_try_from: Option<Type>,
    pub default: bool,
    pub rename_rule: Option<RenameRule>,
    pub metadata: HashMap<String, String>,
}

impl Container {
    pub fn from_input(input: &DeriveInput) -> Result<Self, syn::Error> {
        let mut cont = Container::default();

        let serde_ctx = sdi::Ctxt::new();
        let serde = sdi::attr::Container::from_ast(&serde_ctx, input);
        serde_ctx.check().map_err(|_| {
            syn::Error::new_spanned(&input.ident, "error parsing serde attributes for this type")
        })?;

        cont.tag_type = match serde.tag() {
            sdi::attr::TagType::External => TagType::External,
            sdi::attr::TagType::Internal { tag } => TagType::Internal(tag.clone()),
            sdi::attr::TagType::Adjacent { .. } =>
                return Err(syn::Error::new_spanned(&input.ident, "this type uses the adjacent enum representation, but `jtd_derive` doesn't support it")),
            sdi::attr::TagType::None =>
                return Err(syn::Error::new_spanned(&input.ident, "this type uses the untagged enum representation, but `jtd_derive` doesn't support it")),
        };
        cont.deny_unknown_fields = serde.deny_unknown_fields();
        cont.transparent = serde.transparent();
        cont.type_from = serde.type_from().cloned();
        cont.type_try_from = serde.type_try_from().cloned();
        cont.default = !matches!(serde.default(), sdi::attr::Default::None);
        cont.rename_rule = super::parse_rename_rule(collect_attrs(&input.attrs, SERDE_ATTR_IDENT)?);

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
                    "tag" => {
                        if let Meta::NameValue(v) = p {
                            if let Lit::Str(s) = v.lit {
                                cont.tag_type = TagType::Internal(s.value());
                                Ok(())
                            } else {
                                Err(syn::Error::new_spanned(v.lit, "expected a string literal"))
                            }
                        } else {
                            Err(syn::Error::new_spanned(
                                p,
                                "expected something like `tag = \"...\"`",
                            ))
                        }
                    }
                    "deny_unknown_fields" => {
                        if let Meta::Path(_) = p {
                            cont.deny_unknown_fields = true;
                            Ok(())
                        } else {
                            Err(syn::Error::new_spanned(
                                p,
                                "the `deny_unknown_fields` parameter takes no value",
                            ))
                        }
                    }
                    "transparent" => {
                        if let Meta::Path(_) = p {
                            cont.transparent = true;
                            Ok(())
                        } else {
                            Err(syn::Error::new_spanned(
                                p,
                                "the `transparent` parameter takes no value",
                            ))
                        }
                    }
                    "from" => {
                        if let Meta::NameValue(v) = p {
                            if let Lit::Str(s) = v.lit {
                                cont.type_from = Some(s.parse()?);
                                Ok(())
                            } else {
                                Err(syn::Error::new_spanned(v.lit, "expected a string literal"))
                            }
                        } else {
                            Err(syn::Error::new_spanned(
                                p,
                                "expected something like `from = \"FromType\"`",
                            ))
                        }
                    }
                    "try_from" => {
                        if let Meta::NameValue(v) = p {
                            if let Lit::Str(s) = v.lit {
                                cont.type_try_from = Some(s.parse()?);
                                Ok(())
                            } else {
                                Err(syn::Error::new_spanned(v.lit, "expected a string literal"))
                            }
                        } else {
                            Err(syn::Error::new_spanned(
                                p,
                                "expected something like `try_from = \"FromType\"`",
                            ))
                        }
                    }
                    "rename_all" => {
                        if let Meta::NameValue(v) = p {
                            if let Lit::Str(s) = &v.lit {
                                let rule = RenameRule::from_str(&s.value())
                                    .map_err(|e| syn::Error::new_spanned(v.lit, e))?;
                                cont.rename_rule = Some(rule);
                                Ok(())
                            } else {
                                Err(syn::Error::new_spanned(v.lit, "expected a string literal"))
                            }
                        } else {
                            Err(syn::Error::new_spanned(
                                p,
                                "expected something like `rename_all = \"FromType\"`",
                            ))
                        }
                    }
                    "default" => {
                        if let Meta::Path(_) = p {
                            cont.default = true;
                            Ok(())
                        } else {
                            Err(syn::Error::new_spanned(
                                p,
                                "the `default` parameter takes no value",
                            ))
                        }
                    }
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

                            cont.metadata = metadata;
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

        Ok(cont)
    }
}
