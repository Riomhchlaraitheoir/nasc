use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemTrait};
use quote::ToTokens;
use macros_impl::Args;

#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);
    let service = parse_macro_input!(input as ItemTrait);

    match macros_impl::service(args, service) {
        Ok(tokens) => tokens.into_token_stream(),
        Err(err) => err.to_compile_error(),
    }.into()
}

#[proc_macro]
pub fn method_id(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);

    match macros_impl::method_id(input) {
        Ok(tokens) => tokens.into_token_stream(),
        Err(err) => err.to_compile_error(),
    }.into()
}