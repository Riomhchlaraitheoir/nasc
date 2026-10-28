use crate::parse::parse_service;
use proc_macro2::Ident;
use quote::ToTokens;
use syn::{Expr, ItemTrait, Path, Type, Visibility, WherePredicate};

mod parse;

mod output;

/// The function to invoke the service macro
///
/// # Errors
/// Can return a [`syn::Error`] if it fails to parse the input or rejects some part of the input
pub fn service(args: Args, input: ItemTrait) -> syn::Result<impl ToTokens> {
    parse_service(input, args)
}

struct Service {
    args: Args,
    docs: Vec<Expr>,
    vis: Visibility,
    generic_types: Vec<Ident>,
    generic_bounds: Vec<WherePredicate>,
    name: Ident,
    methods: Vec<Method>,
}

struct Method {
    id: Option<Expr>,
    docs: Vec<Expr>,
    name: Ident,
    args: Vec<(Ident, Param)>,
    ret: Param,
}

#[derive(Debug, PartialEq, Eq)]
enum Param {
    Simple(Type),
    Stream(Type),
    Service(Type),
    Option(Box<Self>),
    Result(Box<Self>, Box<Self>),
    Tuple(Vec<Self>),
}

/// This contains any args in the attribute macro invocation that may affect parsing
// There are no such args for now, but we will keep this just in case tha changes
pub struct Args {
    nasc: Path,
}
