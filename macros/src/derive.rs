mod context;

use proc_macro2::Ident;
use syn::{parse_quote, Attribute, DataStruct, DeriveInput, Fields, ItemImpl};

use self::context::Context;

// TODO: support generics, lifetimes, const generics

pub fn derive(input: DeriveInput) -> ItemImpl {
    let ctx = context::get_context(&input);

    let res = match input.data {
        syn::Data::Struct(s) => derive_struct(&ctx, &input.ident, &input.attrs, s),
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => panic!("jtd-derive does not support unions"),
    };

    res
}

pub fn derive_struct(
    _ctx: &Context,
    ident: &Ident,
    attrs: &[Attribute],
    s: DataStruct,
) -> ItemImpl {
    if let Fields::Named(fields) = s.fields {
        let (idents, types): (Vec<_>, Vec<_>) =
            fields.named.iter().map(|f| (&f.ident, &f.ty)).unzip();

        parse_quote! {
            impl ::jtd_derive::JsonTypedef for #ident {
                fn schema() -> ::jtd_derive::schema::Schema {
                    use ::jtd_derive::JsonTypedef;
                    use ::jtd_derive::schema::{Schema, SchemaType};
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
        }
    } else {
        // TODO: support it if it looks like a newtype (one field)
        panic!("unit and tuple structs are unsupported")
    }
}
