use syn::DeriveInput;

// const ATTR_PATH: &str = "typedef";

pub struct Context {}

pub fn get_context(_input: &DeriveInput) -> Context {
    Context {}
}
