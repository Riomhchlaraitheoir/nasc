use proc_macro2::Ident;
use syn::{parse_quote, Expr, Path, Type, Visibility, WherePredicate};

mod parse;

pub struct Input {
    pub args: Args,
    pub docs: Vec<Expr>,
    pub vis: Visibility,
    pub generic_types: Vec<Ident>,
    pub generic_bounds: Vec<WherePredicate>,
    pub name: Ident,
    pub methods: Vec<Method>,
}

pub struct Method {
    pub id: Expr,
    pub docs: Vec<Expr>,
    pub name: Ident,
    pub args: Vec<(Ident, Param)>,
    pub ret: Param,
}

/// This contains any args in the attribute macro invocation that may affect parsing
pub struct Args {
    pub nasc: Path,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Param {
    Simple(Type),
    Stream(Type),
    Service(Type),
    Option(Box<Self>),
    Result(Box<Self>, Box<Self>),
    Tuple(Vec<Self>),
}

impl Param {
    fn flatten(&mut self) {
        match self {
            Self::Simple(_) | Self::Stream(_) | Self::Service(_) => {}
            Self::Option(param) => {
                param.flatten();
                if let Self::Simple(ty) = &**param {
                    *self = Self::Simple(parse_quote!(Option<#ty>));
                }
            }
            Self::Result(ok, err) => {
                ok.flatten();
                err.flatten();
                if let Self::Simple(ok) = &**ok && let Self::Simple(err) = &**err {
                    *self = Self::Simple(parse_quote!(Result<#ok, #err>));
                }
            }
            Self::Tuple(elements) => {
                for element in elements.iter_mut() {
                    element.flatten();
                }
                let types: Option<Vec<_>> = elements.iter().map(|element| match element {
                    Self::Simple(ty) => Some(ty),
                    _ => None
                }).collect();
                if let Some(types) = types {
                    *self = Self::Simple(parse_quote!((#(#types),*)));
                }
            }
        }
    }
}
