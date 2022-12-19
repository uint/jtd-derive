mod context;

use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{
    parse_quote, DataEnum, DataStruct, DeriveInput, Fields, GenericParam, Generics, Ident, ItemImpl,
};

use self::context::Context;

pub fn derive(input: DeriveInput) -> ItemImpl {
    let ctx = context::get_context(&input);
    let ident = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut impl_generics: Generics = parse_quote! {#impl_generics};
    for param in impl_generics.params.iter_mut() {
        if let GenericParam::Type(ty) = param {
            // We add the `JsonTypedef` bound to every type parameter.
            // This isn't always correct, but it's an okay-ish heuristic.
            ty.bounds.push(parse_quote! { ::jtd_derive::JsonTypedef });
        }
    }

    let res = match input.data {
        syn::Data::Struct(s) => gen_struct_schema(&ctx, &ident, s),
        syn::Data::Enum(e) => gen_enum_schema(&ctx, &ident, e),
        syn::Data::Union(_) => {
            quote_spanned! {ident.span()=> compile_error!("jtd-derive does not support unions")}
        }
    };

    parse_quote! {
        impl #impl_generics ::jtd_derive::JsonTypedef for #ident #ty_generics #where_clause {
            fn schema() -> ::jtd_derive::schema::Schema {
                use ::jtd_derive::JsonTypedef;
                use ::jtd_derive::schema::{Schema, SchemaType};
                #res
            }
        }
    }
}

pub fn gen_struct_schema(_ctx: &Context, ident: &Ident, s: DataStruct) -> TokenStream {
    match s.fields {
        Fields::Named(_) if s.fields.is_empty() => {
            quote_spanned! {ident.span()=> compile_error!("jtd-derive does not support cstruct-like structs")}
        }

        Fields::Named(fields) => {
            let (idents, types): (Vec<_>, Vec<_>) =
                fields.named.iter().map(|f| (&f.ident, &f.ty)).unzip();

            parse_quote! {
                Schema {
                    ty: SchemaType::Properties {
                        properties: [#((stringify!(#idents), <#types as JsonTypedef>::schema())),*].into(),
                        optional_properties: [].into(),
                        additional_properties: true,
                    },
                    ..::jtd_derive::schema::Schema::empty()
                }
            }
        }
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            let ty = &fields.unnamed[0].ty;

            parse_quote! {
                <#ty as JsonTypedef>::schema()
            }
        }
        Fields::Unnamed(_) => {
            quote_spanned! {ident.span()=> compile_error!("jtd-derive only supports tuple structs if they have exactly one field")}
        }
        _ => {
            quote_spanned! {ident.span()=> compile_error!("jtd-derive does not support unit structs")}
        }
    }
}

pub fn gen_enum_schema(_ctx: &Context, ident: &Ident, e: DataEnum) -> TokenStream {
    match enum_kind(&e) {
        EnumKind::UnitLikeVariants => todo!(),
        EnumKind::CstructLikeVariants => todo!(),
        EnumKind::SomeTupleVariants(span) => {
            quote_spanned! {span=> compile_error!("jtd-derive does not support tuple variants")}
        }
        EnumKind::Mixed => {
            quote_spanned! {ident.span()=> compile_error!("jtd-derive requires all enum variants to be of the same kind (unit-like or cstruct-like)")}
        }
        EnumKind::Empty => {
            quote_spanned! {ident.span()=> compile_error!("jtd-derive does not support enums with no variants")}
        }
    }
}

fn enum_kind(e: &DataEnum) -> EnumKind {
    // (named, unit)
    let mut counts = (0, 0);

    for variant in &e.variants {
        match variant.fields {
            Fields::Named(_) => counts.0 += 1,
            Fields::Unit => counts.1 += 1,
            Fields::Unnamed(_) => return EnumKind::SomeTupleVariants(variant.ident.span()),
        }
    }

    match counts {
        (0, 0) => EnumKind::Empty,
        (0, _) => EnumKind::UnitLikeVariants,
        (_, 0) => EnumKind::CstructLikeVariants,
        _ => EnumKind::Mixed,
    }
}

enum EnumKind {
    UnitLikeVariants,
    CstructLikeVariants,
    SomeTupleVariants(Span),
    Mixed,
    Empty,
}
