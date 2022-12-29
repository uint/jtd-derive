use itertools::{Either, Itertools as _};
use serde_derive_internals as sdi;
use syn::{Attribute, DeriveInput, Lit, Meta, NestedMeta};

const ATTR_IDENT: &str = "typedef";

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub no_serde: bool,
    pub tag_type: TagType,
    pub deny_unknown_fields: bool,
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

        let typedef_attrs = input.attrs.iter().filter(|attr| {
            attr.path
                .get_ident()
                .map(|ident| *ident == ATTR_IDENT)
                .unwrap_or(false)
        });

        fn parse_attr(attr: &Attribute) -> Result<Vec<Meta>, syn::Error> {
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
                    "typedef attributes are expected to take this form: #[typedef(...)]",
                ))
            }
        }

        let params = collect_fallible(typedef_attrs.map(parse_attr))?
            .into_iter()
            .flatten();
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
                            "`the `deny_unknown_fields` parameter takes no value",
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
