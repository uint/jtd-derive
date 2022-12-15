mod context;

use syn::{
    parse_quote, DataStruct, DeriveInput, ExprStruct, Fields, GenericParam, Generics, ItemImpl,
};

use self::context::Context;

// TODO: support generics, lifetimes, const generics

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
        syn::Data::Struct(s) => construct_struct(&ctx, s),
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => panic!("jtd-derive does not support unions"),
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

pub fn construct_struct(_ctx: &Context, s: DataStruct) -> ExprStruct {
    if let Fields::Named(fields) = s.fields {
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
    } else {
        // TODO: support it if it looks like a newtype (one field)
        panic!("unit and tuple structs are unsupported")
    }
}
