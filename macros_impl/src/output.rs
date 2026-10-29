use crate::{Method, Param, Service};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Expr, ExprLit, Lit, LitInt};

impl ToTokens for Service {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let nasc = &self.args.nasc;
        let docs = {
            let docs = &self.docs;
            quote! {{
                #(#docs),*
            }}
        };
        let vis = &self.vis;
        let ty_generics = &self.generic_types;
        let where_bounds = &self.generic_bounds;
        let name = &self.name;
        let methods = &self.methods;
        tokens.extend(quote! {
            #nasc::__generate_service!{
                docs = #docs,
                vis = #vis,
                generics = [#(#ty_generics),*],
                where_bounds = [#(#where_bounds),*],
                name = #name,
                methods = [#(#methods)*],
            }
        });
    }
}

impl ToTokens for Method {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = self.id.clone().unwrap_or_else(|| generate_id(&self.name));
        let docs = {
            let docs = &self.docs;
            quote! {{
                #(#docs),*
            }}
        };
        let name = &self.name;
        let args = self.args.iter().map(|(name, param)| {
            let user = param.output();
            quote! {{
                name = #name,
                user = #user,
            }}
        });
        let return_type = self.ret.output();

        let streamed_in = self
            .args
            .iter()
            .flat_map(|(_, param)| param.streamed_types());
        let streamed_out = self.ret.streamed_types();
        let services_in = self
            .args
            .iter()
            .flat_map(|(_, param)| param.services());
        let services_out = self.ret.services();

        tokens.extend(quote! {{
            id = #id,
            docs = #docs,
            name = #name,
            args = [#(#args)*],
            return_type = #return_type,
            streamed_in = [#(#streamed_in),*],
            streamed_out = [#(#streamed_out),*],
            services_in = [#(#services_in),*],
            services_out = [#(#services_out),*],
        }});
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

impl Param {
    fn output(&self) -> TokenStream {
        match self {
            Self::Simple(ty) => quote!((normal: #ty)),
            Self::Stream(ty) => {
                let ty = ty.to_token_stream();
                quote!((stream: #ty))
            }
            Self::Service(ty) => {
                let ty = ty.to_token_stream();
                quote!((service: #ty))
            }
            Self::Option(param) => {
                let param = param.output();
                quote!((option: #param))
            }
            Self::Result(ok, err) => {
                let ok = ok.output();
                let err = err.output();
                quote!((result: #ok, #err))
            }
            Self::Tuple(params) => {
                let params = params.iter().enumerate().map(|(i, p)| {
                    let name = format_ident!("_{i}");
                    let param = p.output();
                    quote!(#name: #param)
                });
                quote!((tuple: #(#params),*))
            }
        }
    }

    fn streamed_types(&self) -> Vec<TokenStream> {
        let mut types = Vec::new();
        self.streamed_types_inner(&mut types);
        types
    }

    fn streamed_types_inner(&self, types: &mut Vec<TokenStream>) {
        match self {
            Self::Simple(_) | Self::Service(_) => {}
            Self::Stream(ty) => {
                types.push(ty.to_token_stream());
            }
            Self::Option(value) => value.streamed_types_inner(types),
            Self::Result(ok, err) => {
                ok.streamed_types_inner(types);
                err.streamed_types_inner(types);
            }
            Self::Tuple(params) => {
                for param in params {
                    param.streamed_types_inner(types);
                }
            }
        }
    }

    fn services(&self) -> Vec<TokenStream> {
        let mut types = Vec::new();
        self.services_inner(&mut types);
        types
    }

    fn services_inner(&self, types: &mut Vec<TokenStream>) {
        match self {
            Self::Simple(_) | Self::Stream(_) => {}
            Self::Service(ty) => {
                types.push(ty.to_token_stream());
            }
            Self::Option(value) => value.streamed_types_inner(types),
            Self::Result(ok, err) => {
                ok.streamed_types_inner(types);
                err.streamed_types_inner(types);
            }
            Self::Tuple(params) => {
                for param in params {
                    param.streamed_types_inner(types);
                }
            }
        }
    }
}
