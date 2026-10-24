use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemTrait};
use quote::ToTokens;
use macros_impl::Args;

#[proc_macro_attribute]
pub fn rpc(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);
    let rpc = parse_macro_input!(input as ItemTrait);

    match macros_impl::rpc(args, rpc) {
        Ok(tokens) => tokens.into_token_stream(),
        Err(err) => err.to_compile_error(),
    }.into()
}