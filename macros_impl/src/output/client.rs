use crate::output::{
    ClientImpl, ClientImplFn, ClientStruct, ClientTrait, ClientTraitFn, GenericWhere, Names,
};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

impl ToTokens for ClientTrait<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            names: Names { client_trait, .. },
            methods,
            generic_types,
            generic_bounds,
        } = self;
        let where_clause = GenericWhere(generic_bounds);
        tokens.extend(quote! {
            pub trait #client_trait<#(#generic_types),*>: Send + Sync #where_clause {
                type Format: imports::Format;

                #(#methods)*
            }
        });
    }
}

impl ToTokens for ClientTraitFn<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            name,
            docs,
            arg_names,
            arg_types,
            return_type,
        } = self;
        let arg_types = arg_types.iter().map(|p| &p.client_input);
        let return_type = &return_type.client_output;
        tokens.extend(quote! {
            #(#[doc = #docs])*
            ///
            /// Returns a future builder that allows certain options to be set before `await`-ing
            ///
            /// see [`ClientResponseFuture`](imports::ClientResponseFuture)
            fn #name(&self, #(#arg_names: #arg_types),*) -> imports::ClientResponseFuture<impl imports::Future<Output=Result<#return_type, imports::ClientError>> + Send>;
        });
    }
}

impl ToTokens for ClientStruct<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            names:
                Names {
                    service,
                    client_struct,
                    ..
                },
            generic_types,
            generic_bounds,
        } = self;
        let where_clause = GenericWhere(generic_bounds);
        tokens.extend(quote! {
            #[derive(Debug, Clone, PartialEq)]
            pub struct #client_struct<_Format, #(#generic_types),*> #where_clause {
                client: imports::Client<#service<#(#generic_types),*>>,
                format: _Format,
                _generics: imports::PhantomData<fn() -> (#(#generic_types,)*)>,
            }
        });
    }
}

impl ToTokens for ClientImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            names:
                Names {
                    client_trait,
                    client_struct,
                    ..
                },
            docs,
            write_types,
            read_types,
            methods,
            generic_types,
            generic_bounds,
        } = self;
        let where_clause = if read_types.is_empty() && write_types.is_empty() {
            TokenStream::new()
        } else {
            quote! {
            where
                #(_Format: imports::ReadFormat<#read_types>,)*
                #(_Format: imports::WriteFormat<#write_types>,)*
                #(#generic_bounds,)*
            }
        };
        tokens.extend(quote! {
            #(#[doc = #docs])*
            ///
            /// This is the async client for the service, it produces requests from method calls
            /// and sends the requests with the given [client](imports::Client) before returning
            /// the response
            ///
            /// The return value is always wrapped in a result: `Result<T, ClientError<_Conn::SendError>>`
            /// where `T` is the service return value
            impl<_Format, #(#generic_types),*> #client_trait<#(#generic_types),*> for #client_struct<_Format, #(#generic_types),*> #where_clause {
                type Format = _Format;

                #(#methods)*
            }
        });
    }
}

impl ToTokens for ClientImplFn<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            name,
            id,
            docs,
            arg_names,
            arg_types,
            return_type,
            prepare_input,
            prepare_output,
        } = self;
        let arg_types = arg_types.iter().map(|p| &p.client_input);
        let return_type_transport = &return_type.transport;
        let return_type = &return_type.client_output;
        tokens.extend(quote! {
            #(#[doc = #docs])*
            fn #name(&self, #(#arg_names: #arg_types),*) -> imports::ClientResponseFuture<impl Future<Output=Result<#return_type, imports::ClientError>> + Send> {
                let request = {
                    let format = self.format.clone();
                    let manager = self.client.service_manager();
                    let guard = self.client.guard.clone();
                    async move {
                        let value = (#({
                            let value = #arg_names;
                            #prepare_output
                        },)*);
                        format.write(value).map_err(imports::ClientError::Serialise)
                    }
                };
                let response = {
                    let format = self.format.clone();
                    let manager = self.client.service_manager();
                    let guard = self.client.guard.clone();
                    async move |response: Vec<u8>| -> Result<_, imports::ClientError> {
                        let value: #return_type_transport = format.read(&response).map_err(imports::ClientError::Deserialise)?;
                        Ok(#prepare_input)
                    }
                };
                self.client.send(request, imports::MethodID::_new(#id), response)
            }
        });
    }
}
