use std::collections::HashMap;

use syn::Type;

use super::context::FieldCtx;

pub struct Field {
    pub ty: Type,
    pub ident: String,
    pub meta: HashMap<String, String>,
}

impl Field {
    pub fn from_syn_field(f: &syn::Field) -> Result<Self, syn::Error> {
        let ctx = FieldCtx::from_input(f)?;

        Ok(Self {
            ty: f.ty.clone(),
            ident: f.ident.as_ref().map(|i| i.to_string()).unwrap(),
            meta: ctx.metadata,
        })
    }
}
