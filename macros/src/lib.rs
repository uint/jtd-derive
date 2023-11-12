//! Proc macros for `jtd-derive`.
//!
//! This crate is tightly tied to the `jtd-derive` crate. It's **not meant to be
//! used as a direct dependency** in your project. Instead, please depend on the
//! [`jtd-derive`](https://docs.rs/jtd-derive) crate, which provides documentation
//! and access to the derive macro.

mod derive;
pub(crate) mod iter_ext;

use quote::ToTokens;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(JsonTypedef, attributes(typedef))]
pub fn query_responses_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = match derive::derive(input) {
        Ok(item_impl) => item_impl.into_token_stream(),
        Err(e) => e.into_compile_error(),
    };

    proc_macro::TokenStream::from(expanded)
}
