use crate::{Args, Method, Param, Rpc};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Type, parse_quote};

impl ToTokens for Rpc {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let trait_rpc = &self.args.trait_rpc;
        let args = &self.args;
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
            #trait_rpc::__generate_rpc!{
                args = #args,
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

impl ToTokens for Args {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let serde = &self.serde;
        tokens.extend(quote! {{
            serde = #serde,
        }});
    }
}

impl ToTokens for Method {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let docs = {
            let docs = &self.docs;
            quote! {{
                    #(#docs),*
                }}
        };
        let name = &self.name;
        let args = self.args.iter().map(|(name, param)| {
            let user = param.normal();
            let transport = param.for_transport();
            quote! {{
                name = #name,
                user = (#user),
                transport = #transport,
            }}
        });
        let return_type = {
            let user = self.ret.normal();
            let transport = self.ret.for_transport();
            quote! {{
                user = (#user),
                transport = #transport,
            }}
        };

        let streamed_in = self.args.iter().flat_map(|(_, param)| {
            param.streamed_types()
        });
        let streamed_out = self.ret.streamed_types();

        tokens.extend(quote! {{
            docs = #docs,
            name = #name,
            args = [#(#args)*],
            return_type = #return_type,
            streamed_in = [#(#streamed_in),*],
            streamed_out = [#(#streamed_out),*],
        }});
    }
}

impl Param {
    fn normal(&self) -> TokenStream {
        match self {
            Self::Simple(ty) => ty.to_token_stream(),
            Self::Stream(ty) => {
                let ty = ty.to_token_stream();
                quote! { impl futures::stream::Stream<Item=#ty> }
            }
            Self::Option(param) => {
                let param = param.normal();
                quote! { Option<#param> }
            }
            Self::Result(ok, err) => {
                let ok = ok.normal();
                let err = err.normal();
                quote! { Result<#ok, #err> }
            }
            Self::Tuple(params) => {
                let params = params.iter().map(Self::normal);
                quote! { ( #(#params,)* ) }
            }
        }
    }
    fn for_transport(&self) -> Type {
        match self {
            Self::Simple(ty) => ty.clone(),
            Self::Stream(_) => parse_quote! { StreamID },
            Self::Option(param) => {
                let param = param.for_transport();
                parse_quote! { Option<#param> }
            }
            Self::Result(ok, err) => {
                let ok = ok.for_transport();
                let err = err.for_transport();
                parse_quote! { Result<#ok, #err> }
            }
            Self::Tuple(params) => {
                let params = params.iter().map(Self::for_transport);
                parse_quote! { ( #(#params,)* ) }
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
            Self::Simple(_) => {}
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
}
