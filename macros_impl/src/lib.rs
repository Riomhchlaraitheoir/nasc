use crate::parse::parse_rpc;
use proc_macro2::Ident;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Expr, ItemTrait, LitStr, Path, Token, Type, Visibility, WherePredicate, parse_quote};

mod parse;

mod output;

/// The function to invoke the rpc macro
///
/// # Errors
/// Can return a [`syn::Error`] if it fails to parse the input or rejects some part of the input
pub fn rpc(args: Args, input: ItemTrait) -> syn::Result<impl ToTokens> {
    parse_rpc(input, args)
}

struct Rpc {
    args: Args,
    docs: Vec<Expr>,
    vis: Visibility,
    generic_types: Vec<Ident>,
    generic_bounds: Vec<WherePredicate>,
    name: Ident,
    methods: Vec<Method>,
}

struct Method {
    docs: Vec<Expr>,
    name: Ident,
    args: Vec<(Ident, Param)>,
    ret: Param,
}

#[derive(Debug, PartialEq, Eq)]
enum Param {
    Simple(Type),
    Stream(Type),
    Option(Box<Self>),
    Result(Box<Self>, Box<Self>),
    Tuple(Vec<Self>),
}

/// This contains any args in the attribute macro invocation that may affect parsing
// There are no such args for now, but we will keep this just in case tha changes
pub struct Args {
    trait_rpc: Path,
    serde: LitStr,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut trait_rpc = None;
        let mut serde = None;
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _eq_token: Token![=] = input.parse()?;
            match key.to_string().as_str() {
                "trait_rpc" => {
                    trait_rpc = Some(input.parse()?);
                }
                "serde" => {
                    serde = Some(input.parse()?);
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        key,
                        format!("unknown arg: {other}"),
                    ));
                }
            }
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }
        let trait_rpc: Path = trait_rpc.unwrap_or_else(|| parse_quote!(::trait_rpc));
        let serde = serde.unwrap_or_else(|| {
            let span = trait_rpc.span();
            let trait_rpc = trait_rpc.clone().to_token_stream().to_string().replace(' ', "");
            LitStr::new(&format!("{trait_rpc}::serde"), span)
        });
        Ok(Self {
            trait_rpc,
            serde,
        })
    }
}
