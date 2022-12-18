mod context;

use proc_macro2::TokenStream;
use syn::{
    parse_quote, DataEnum, DataStruct, DeriveInput, Fields, GenericParam, Generics, ItemImpl,
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
        syn::Data::Struct(s) => gen_struct_schema(&ctx, s),
        syn::Data::Enum(e) => gen_enum_schema(&ctx, e),
        syn::Data::Union(_) => parse_quote! {compile_error!("jtd-derive does not support unions")},
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

pub fn gen_struct_schema(_ctx: &Context, s: DataStruct) -> TokenStream {
    match s.fields {
        Fields::Named(_) if s.fields.len() == 0 => {
            parse_quote! {compile_error!("empty cstruct-like structs are unsupported")}
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
            parse_quote! {compile_error!("tuple structs are only supported if they have exactly one field")}
        }
        _ => parse_quote! {compile_error!("unit structs are unsupported")},
    }
}

pub fn gen_enum_schema(_ctx: &Context, e: DataEnum) -> TokenStream {
    match enum_kind(&e) {
        EnumKind::UnitLikeVariants => todo!(),
        EnumKind::CstructLikeVariants => todo!(),
        EnumKind::SomeTupleVariants => {
            parse_quote! {compile_error!("tuple variants are unsupported")}
        }
        EnumKind::Mixed => {
            parse_quote! {compile_error!("all enum variants must be of the same kind (unit-like or cstruct-like)")}
        }
        EnumKind::Empty => {
            parse_quote! {compile_error!("enums with no variants are unsupported")}
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
            Fields::Unnamed(_) => return EnumKind::SomeTupleVariants,
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
    SomeTupleVariants,
    Mixed,
    Empty,
}
