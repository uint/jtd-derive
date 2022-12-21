mod derive;

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
