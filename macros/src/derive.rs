mod context;
pub mod field;

use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use serde_derive_internals::attr::RenameRule;
use syn::{
    parse_quote, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, GenericParam, Generics,
    Ident, ItemImpl,
};

use crate::{derive::field::Field, iter_ext::IterExt};

use self::context::Container;

pub fn derive(input: DeriveInput) -> Result<ItemImpl, syn::Error> {
    let ctx = context::Container::from_input(&input)?;

    let ident = input.ident;

    let (impl_generics_no_infer, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut impl_generics: Generics = parse_quote! {#impl_generics_no_infer};
    for param in impl_generics.params.iter_mut() {
        if let GenericParam::Type(ty) = param {
            // We add the `JsonTypedef` bound to every type parameter.
            // This isn't always correct, but it's an okay-ish heuristic.
            ty.bounds.push(parse_quote! { ::jtd_derive::JsonTypedef });
        }
    }

    let type_params = input.generics.type_params().map(|p| &p.ident);
    let const_params = input.generics.const_params().map(|p| &p.ident);

    let names_impl = quote! {
        fn names() -> ::jtd_derive::Names {
            ::jtd_derive::Names {
                short: stringify!(#ident),
                long: concat!(module_path!(), "::", stringify!(#ident)),
                nullable: false,
                type_params: [#(#type_params::names()),*].into(),
                const_params: [#(#const_params.to_string()),*].into(),
            }
        }
    };

    match (&ctx.type_from, &ctx.type_try_from) {
        (None, None) => {}
        (Some(ty), None) => {
            return Ok(parse_quote! {
                impl #impl_generics_no_infer ::jtd_derive::JsonTypedef for #ident #ty_generics #where_clause {
                    fn schema(gen: &mut ::jtd_derive::Generator) -> ::jtd_derive::schema::Schema {
                        <#ty as ::jtd_derive::JsonTypedef>::schema(gen)
                    }

                    fn referenceable() -> bool {
                        <#ty as ::jtd_derive::JsonTypedef>::referenceable()
                    }

                    fn names() -> ::jtd_derive::Names {
                        <#ty as ::jtd_derive::JsonTypedef>::names()
                    }
                }
            });
        }
        (None, Some(ty)) => {
            return Ok(parse_quote! {
                impl #impl_generics_no_infer ::jtd_derive::JsonTypedef for #ident #ty_generics #where_clause {
                    fn schema(gen: &mut ::jtd_derive::Generator) -> ::jtd_derive::schema::Schema {
                        <#ty as ::jtd_derive::JsonTypedef>::schema(gen)
                    }

                    fn referenceable() -> bool {
                        true
                    }

                    #names_impl
                }
            });
        }
        (Some(_), Some(_)) => {
            return Err(syn::Error::new_spanned(
                ident,
                "can't set both `#[typedef(from = \"...\")]` and `#[typedef(try_from = \"...\")]`",
            ));
        }
    }

    let res = match input.data {
        syn::Data::Struct(s) => gen_struct_schema(&ctx, &ident, s)?,
        syn::Data::Enum(e) => gen_enum_schema(&ctx, &ident, e)?,
        syn::Data::Union(_) => {
            quote_spanned! {ident.span()=> compile_error!("jtd-derive does not support unions")}
        }
    };
    let meta = gen_metadata(&ctx.metadata);

    let res = quote! { {
        let mut schema = #res;
        schema.metadata.extend(#meta);
        schema
    } };

    Ok(parse_quote! {
        impl #impl_generics ::jtd_derive::JsonTypedef for #ident #ty_generics #where_clause {
            fn schema(gen: &mut ::jtd_derive::Generator) -> ::jtd_derive::schema::Schema {
                use ::jtd_derive::JsonTypedef;
                use ::jtd_derive::schema::{Schema, SchemaType};
                #res
            }

            fn referenceable() -> bool {
                true
            }

            #names_impl
        }
    })
}

fn gen_struct_schema(
    ctx: &Container,
    ident: &Ident,
    s: DataStruct,
) -> Result<TokenStream, syn::Error> {
    match s.fields {
        Fields::Named(_) if s.fields.is_empty() => Err(syn::Error::new_spanned(
            ident,
            "jtd-derive does not support empty cstruct-like structs",
        )),
        Fields::Named(fields) if s.fields.len() == 1 && ctx.transparent => {
            let ty = &fields.named[0].ty;

            Ok(parse_quote! {
                gen.sub_schema::<#ty>()
            })
        }
        Fields::Named(fields) => {
            if ctx.transparent {
                Err(syn::Error::new_spanned(
                    ident,
                    "#[typedef(transparent)] requires struct to have exactly one field",
                ))
                //}
            } else {
                gen_named_fields(ctx, &fields, ctx.rename_rule)
            }
        }
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            let ty = &fields.unnamed[0].ty;

            Ok(parse_quote! {
                gen.sub_schema::<#ty>()
            })
        }
        Fields::Unnamed(_) => Err(syn::Error::new_spanned(
            ident,
            "jtd-derive only supports tuple structs if they have exactly one field",
        )),
        _ => Err(syn::Error::new_spanned(
            ident,
            "jtd-derive does not support unit structs",
        )),
    }
}

fn gen_enum_schema(
    ctx: &Container,
    ident: &Ident,
    enu: DataEnum,
) -> Result<TokenStream, syn::Error> {
    if ctx.transparent {
        return Err(syn::Error::new_spanned(
            ident,
            "#[typedef(transparent)] is not allowed on an enum",
        ));
    }

    if ctx.default {
        return Err(syn::Error::new_spanned(
            ident,
            "#[typedef(default)] is not allowed on an enum",
        ));
    }

    match enum_kind(ident, &enu)? {
        EnumKind::UnitVariants => {
            let mut idents: Vec<_> = enu.variants.iter().map(|v| v.ident.to_string()).collect();
            if let Some(rule) = ctx.rename_rule {
                for ident in idents.iter_mut() {
                    *ident = rule.apply_to_variant(ident);
                }
            }

            let enum_schema = parse_quote! {
                Schema {
                    ty: SchemaType::Enum {
                        r#enum: [#(#idents),*].into(),
                    },
                    ..::jtd_derive::schema::Schema::default()
                }
            };

            match &ctx.tag_type {
                context::TagType::External => Ok(enum_schema),
                context::TagType::Internal(tag) => Ok(parse_quote! {
                    Schema {
                        ty: SchemaType::Properties {
                            properties: [
                                (#tag, #enum_schema)
                            ].into(),
                            additional_properties: true,
                            optional_properties: [].into(),
                        },
                        ..::jtd_derive::schema::Schema::default()
                    }
                }),
            }
        }
        EnumKind::StructVariants => {
            let tag = match &ctx.tag_type {
                context::TagType::External => {
                    return Err(syn::Error::new_spanned(
                        ident,
                        "jtd-derive requires an enum with struct variants to have a tag",
                    ));
                }
                context::TagType::Internal(t) => t,
            };

            let (mut idents, variants): (Vec<_>, Vec<_>) = enu
                .variants
                .iter()
                .map(|v| {
                    (
                        v.ident.to_string(),
                        gen_named_fields(ctx, unwrap_fields_named(&v.fields), None),
                    )
                })
                .unzip();
            let variants: Vec<_> = variants.into_iter().collect_fallible()?;
            if let Some(rule) = ctx.rename_rule {
                for ident in idents.iter_mut() {
                    *ident = rule.apply_to_variant(ident);
                }
            }

            Ok(parse_quote! {
                Schema {
                    ty: SchemaType::Discriminator {
                        discriminator: #tag,
                        mapping: [#((#idents, #variants)),*].into(),
                    },
                    ..::jtd_derive::schema::Schema::default()
                }
            })
        }
    }
}

fn gen_metadata(meta: &HashMap<String, String>) -> TokenStream {
    let keys = meta.keys();
    let values = meta.values();
    quote! { [#((#keys, #values.parse::<::serde_json::Value>().unwrap())),*] }
}

fn gen_named_fields(
    ctx: &Container,
    fields: &FieldsNamed,
    rename_rule: Option<RenameRule>,
) -> Result<TokenStream, syn::Error> {
    let fields: Vec<_> = fields
        .named
        .iter()
        .map(Field::from_syn_field)
        .collect_fallible()?;

    let mut idents: Vec<_> = fields.iter().map(|f| f.ident.clone()).collect();
    let types: Vec<_> = fields.iter().map(|f| f.ty.clone()).collect();
    let metas: Vec<_> = fields.into_iter().map(|f| gen_metadata(&f.meta)).collect();

    if let Some(rule) = rename_rule {
        for ident in idents.iter_mut() {
            *ident = rule.apply_to_field(&ident.to_string());
        }
    }

    let expanded_fields = quote! {#((#idents, {
        let mut schema = gen.sub_schema::<#types>();
        schema.metadata.extend(#metas);
        schema
    })),*};

    let additional = !ctx.deny_unknown_fields;

    let (prop, optional) = if ctx.default {
        (quote! {[].into()}, quote! {[#expanded_fields].into()})
    } else {
        (quote! {[#expanded_fields].into()}, quote! {[].into()})
    };

    Ok(parse_quote! {
        Schema {
            ty: SchemaType::Properties {
                properties: #prop,
                optional_properties: #optional,
                additional_properties: #additional,
            },
            ..::jtd_derive::schema::Schema::default()
        }
    })
}

fn unwrap_fields_named(fields: &Fields) -> &FieldsNamed {
    if let Fields::Named(named) = fields {
        named
    } else {
        // this branch should never be reached, so it being a panic and not
        // a quoted compile_error is OK
        panic!("expected named fields")
    }
}

fn enum_kind(ident: &Ident, e: &DataEnum) -> Result<EnumKind, syn::Error> {
    let (mut named, mut unit) = (None, None);

    for variant in &e.variants {
        match variant.fields {
            Fields::Named(_) => {
                named = Some(variant);
                if unit.is_some() {
                    break;
                }
            }
            Fields::Unit => {
                unit = Some(variant);
                if named.is_some() {
                    break;
                }
            }
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    variant,
                    "Typedef can't support tuple variants",
                ))
            }
        }
    }

    match (named, unit) {
        (None, None) => Err(syn::Error::new_spanned(
            ident,
            "jtd-derive does not support empty enums",
        )),
        (None, Some(_)) => Ok(EnumKind::UnitVariants),
        (Some(_), None) => Ok(EnumKind::StructVariants),
        (Some(named), Some(unit)) => {
            let mut err = syn::Error::new_spanned(
                ident,
                "Typedef can't support enums with a mix of unit and struct variants",
            );

            // TODO: if the output looks like independent errors, we probably want
            // to scratch the two errors below. probably
            err.combine(syn::Error::new_spanned(
                unit,
                format!("here's a unit variant of `{}`", ident),
            ));
            err.combine(syn::Error::new_spanned(
                named,
                format!("here's a struct variant of `{}`", ident),
            ));

            Err(err)
        }
    }
}

enum EnumKind {
    // the enum only has unit variants
    UnitVariants,
    // the enum only has struct variants
    StructVariants,
}
