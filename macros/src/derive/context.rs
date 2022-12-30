use itertools::{Either, Itertools as _};
use sdi::attr::RenameRule;
use serde_derive_internals as sdi;
use syn::{Attribute, DeriveInput, Lit, Meta, MetaNameValue, NestedMeta, Type};

const ATTR_IDENT: &str = "typedef";
const SERDE_ATTR_IDENT: &str = "serde";

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
}

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

fn collect_fallible<T>(
    data: impl Iterator<Item = Result<T, syn::Error>>,
) -> Result<Vec<T>, syn::Error> {
    let (good_data, errors): (Vec<_>, Vec<_>) = data.partition_map(|r| match r {
        Ok(v) => Either::Left(v),
        Err(v) => Either::Right(v),
    });

    if let Some(e) = errors.into_iter().reduce(|mut l, r| {
        l.combine(r);
        l
    }) {
        Err(e)
    } else {
        Ok(good_data)
    }
}

impl Container {
    pub fn from_input(input: &DeriveInput) -> Result<Container, syn::Error> {
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
        cont.rename_rule = parse_rename_rule(collect_attrs(input, SERDE_ATTR_IDENT)?);

        let params = collect_attrs(input, ATTR_IDENT)?;
        collect_fallible(params.map(|p| {
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
                _ => Err(syn::Error::new_spanned(
                    p.path(),
                    "unknown jtd-derive parameter",
                )),
            }
        }))?;

        Ok(cont)
    }
}

fn collect_attrs(
    input: &DeriveInput,
    path: &str,
) -> Result<impl Iterator<Item = Meta>, syn::Error> {
    let attrs = input.attrs.iter().filter(|attr| {
        attr.path
            .get_ident()
            .map(|ident| *ident == path)
            .unwrap_or(false)
    });

    let parse_attr = |attr: &Attribute| -> Result<Vec<Meta>, syn::Error> {
        let meta = attr.parse_meta()?;
        if let Meta::List(l) = meta {
            let inner_metas = collect_fallible(l.nested.into_iter().map(|n| {
                if let NestedMeta::Meta(m) = n {
                    Ok(m)
                } else {
                    Err(syn::Error::new_spanned(n, "literals are not allowed here"))
                }
            }))?;

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

    Ok(collect_fallible(attrs.map(parse_attr))?
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
