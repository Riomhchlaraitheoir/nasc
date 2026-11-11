pub use crate::input::Args;
use crate::input::Input;
use crate::output::Output;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::spanned::Spanned;
use syn::{ItemTrait, Path, PathArguments};

mod convert;
mod input;
mod output;
#[cfg(test)]
mod tests;

/// The function to invoke the service macro
///
/// # Errors
/// Can return a [`syn::Error`] if it fails to parse the input or rejects some part of the input
pub fn service(args: Args, input: ItemTrait) -> syn::Result<impl ToTokens> {
    let input = Input::parse(input, args)?;
    let output = Output::from(&input);
    Ok(output.into_token_stream())
}

pub fn method_id(mut input: Path) -> syn::Result<TokenStream> {
    let method_name = input
        .segments
        .pop()
        .ok_or_else(|| syn::Error::new(input.span(), "Empty method specifier"))?;
    let PathArguments::None = method_name.arguments else {
        return Err(syn::Error::new(method_name.arguments.span(), "unexpected path arguments"))
    };
    let method_name = method_name.ident;
    input.segments.pop_punct();
    let service = input;
    let method_id = format_ident!("METHOD_ID_{}", ident_ccase!(upper_snake, method_name));
    Ok(quote! {
        const {
            const fn _assert_is_service<T: ::nasc::Service>() {}
            _assert_is_service::<#service>();
            <#service>::#method_id
        }
    })
}
