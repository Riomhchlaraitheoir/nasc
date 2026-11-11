use crate::input::{Args, Input, Method, Param};
use proc_macro2::Ident;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::token::Colon;
use syn::{parse_quote, Attribute, Expr, ExprLit, FnArg, GenericArgument, GenericParam, ItemTrait, Lit, LitInt, Meta, MetaNameValue, Pat, PatIdent, Path, PathArguments, PathSegment, PredicateType, ReturnType, Safety, Token, TraitItem, TraitItemFn, Type, TypeParamBound, WherePredicate};

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut nasc = None;
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _eq_token: Token![=] = input.parse()?;
            match key.to_string().as_str() {
                "nasc" => {
                    nasc = Some(input.parse()?);
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
        let nasc: Path = nasc.unwrap_or_else(|| parse_quote!(::nasc));
        Ok(Self {
            nasc,
        })
    }
}

impl Input {
    pub fn parse(input: ItemTrait, args: Args) -> syn::Result<Self> {
        let mut methods = vec![];
        for item in input.items {
            if let TraitItem::Fn(item) = item {
                methods.push(method(item)?);
            }
        }
        if !input.supertraits.is_empty() {
            return Err(syn::Error::new_spanned(
                input.supertraits,
                "supertraits are not supported",
            ));
        }
        let docs = input.attrs.iter().filter_map(docs).collect();
        let mut generic_types = Vec::new();
        let mut generic_bounds: Vec<_> = input
            .generics
            .where_clause
            .map_or_default(|w| w.predicates.into_iter().collect());
        for generic in input.generics.params {
            let GenericParam::Type(ty) = generic else {
                return Err(syn::Error::new(
                    generic.span(),
                    "Only generic types are supported",
                ));
            };
            let ident = ty.ident;
            if let Some((_eq, default)) = ty.default {
                return Err(syn::Error::new(
                    default.span(),
                    "Default is not supported",
                ));
            }
            if !ty.bounds.is_empty() {
                generic_bounds.push(WherePredicate::Type(PredicateType {
                    attrs: vec![],
                    lifetimes: None,
                    bounded_ty: parse_quote!(#ident),
                    colon_token: Colon::default(),
                    bounds: ty.bounds,
                }));
            }
            generic_types.push(ident);
        }
        Ok(Self {
            args,
            docs,
            vis: input.vis,
            generic_types,
            generic_bounds,
            name: input.ident,
            methods,
        })
    }
}

fn method(item: TraitItemFn) -> syn::Result<Method> {
    if let Some(default) = item.default {
        return Err(syn::Error::new_spanned(
            default,
            "default fn is not supported",
        ));
    }
    if let Some(con) = item.sig.constness {
        return Err(syn::Error::new_spanned(con, "const fn is not supported"));
    }
    if let Safety::Unsafe(unsafety) = item.sig.safety {
        return Err(syn::Error::new(
            unsafety.span(),
            "unsafe fn is not supported",
        ));
    }
    let name = item.sig.ident.clone();
    let mut args = Vec::with_capacity(item.sig.inputs.len());
    for arg in &item.sig.inputs {
        match arg {
            FnArg::Receiver(s) => {
                return Err(syn::Error::new_spanned(s, "self receiver is not supported"));
            }
            FnArg::Typed(arg) => {
                let Pat::Ident(PatIdent { ident, .. }) = &*arg.pat else {
                    return Err(syn::Error::new(
                        arg.pat.span(),
                        "pattern arguments are not supported",
                    ));
                };
                args.push((ident.clone(), Param::try_from(&*arg.ty)?));
            }
        }
    }
    let ret = return_type(item.sig.output)?;
    let docs = item.attrs.iter().filter_map(docs).collect();
    let id = item.attrs.iter().find_map(id);
    let id = id.unwrap_or_else(|| generate_id(&name));
    Ok(Method {
        id,
        docs,
        name,
        args,
        ret,
    })
}

fn return_type(output: ReturnType) -> syn::Result<Param> {
    match output {
        ReturnType::Default => Ok(Param::Simple(parse_quote! {()})),
        ReturnType::Type(_, ty) => Param::try_from(ty.as_ref()),
    }
}

fn docs(attr: &Attribute) -> Option<Expr> {
    if let Meta::NameValue(MetaNameValue { path, value, .. }) = &attr.meta {
        if path.is_ident("doc") {
            Some(value.clone())
        } else {
            None
        }
    } else {
        None
    }
}

fn id(attr: &Attribute) -> Option<Expr> {
    if let Meta::NameValue(MetaNameValue { path, value, .. }) = &attr.meta {
        if path.is_ident("id") {
            Some(value.clone())
        } else {
            None
        }
    } else {
        None
    }
}

impl TryFrom<&Type> for Param {
    type Error = syn::Error;
    fn try_from(ty: &Type) -> Result<Self, Self::Error> {
        let type_span = ty.span();
        let mut p = match &ty {
            Type::ImplTrait(ty) => {
                if ty.bounds.len() == 1
                    && let TypeParamBound::Trait(t) = &ty.bounds[0]
                    && t.path.segments.len() == 1
                {
                    let PathSegment { ident, arguments } = &t.path.segments[0];
                    if ident == "Stream" {
                        if let PathArguments::AngleBracketed(args) = arguments
                            && args.args.len() == 1
                            && let GenericArgument::AssocType(ty) = &args.args[0]
                            && ty.ident == "Item"
                        {
                            Ok(Self::Stream(ty.ty.clone()))
                        } else {
                            Err(syn::Error::new(
                                type_span,
                                "Stream must declare Item type as: `impl Stream<Item = <item>>`",
                            ))
                        }
                    } else if ident == "Service" {
                        if let PathArguments::AngleBracketed(args) = arguments
                            && args.args.len() == 1
                            && let GenericArgument::Type(ty) = &args.args[0]
                        {
                            Ok(Self::Service(ty.clone()))
                        } else {
                            Err(syn::Error::new(
                                type_span,
                                "Stream must declare Item type as: `impl Stream<Item = <item>>`",
                            ))
                        }
                    } else {
                        Err(syn::Error::new(type_span, "unsupported impl type"))
                    }
                } else {
                    Err(syn::Error::new(
                        type_span,
                        format!("unsupported impl type: {}", ty.to_token_stream()),
                    ))
                }
            }
            Type::Tuple(tuple) => Ok(Self::Tuple(
                tuple
                    .elems
                    .iter()
                    .map(Self::try_from)
                    .collect::<Result<_, _>>()?,
            )),
            Type::Path(ty) => Ok(
                if ty.path.segments.len() == 1
                    && ty.path.segments[0].ident == "Option"
                    && let PathArguments::AngleBracketed(args) = &ty.path.segments[0].arguments
                    && args.args.len() == 1
                    && let GenericArgument::Type(ty) = &args.args[0]
                {
                    Self::Option(Box::new(Self::try_from(ty)?))
                } else if ty.path.segments.len() == 1
                    && ty.path.segments[0].ident == "Result"
                    && let PathArguments::AngleBracketed(args) = &ty.path.segments[0].arguments
                    && args.args.len() == 2
                    && let GenericArgument::Type(ok) = &args.args[0]
                    && let GenericArgument::Type(err) = &args.args[1]
                {
                    Self::Result(
                        Box::new(Self::try_from(ok)?),
                        Box::new(Self::try_from(err)?),
                    )
                } else {
                    Self::Simple(Type::Path(ty.clone()))
                },
            ),
            Type::Paren(ty) => Self::try_from(ty.elem.as_ref()),
            _ => Err(syn::Error::new(
                ty.span(),
                format!("unsupported type: {}", ty.to_token_stream()),
            )),
        }?;
        p.flatten();
        Ok(p)
    }
}

fn generate_id(name: &Ident) -> Expr {
    let span = name.span();
    let name = name.to_string();
    let bytes = name.as_bytes();
    let mut hash = 2_166_136_261;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= u32::from(bytes[i]);
        hash = hash.wrapping_mul(16_777_619);
        i += 1;
    }
    Expr::Lit(ExprLit {
        attrs: vec![],
        lit: Lit::Int(LitInt::new(&format!("{hash}_u32"), span)),
    })
}
