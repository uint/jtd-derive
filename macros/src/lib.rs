mod derive;

use quote::ToTokens;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(JsonTypedef, attributes(serde, typedef))]
pub fn query_responses_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = derive::derive(input).into_token_stream();

    proc_macro::TokenStream::from(expanded)
}
