use crate::output::{
    GenericWhere, HandlerMatchArm, Names, ServerHandler, ServerHandlerImpl, ServerTrait,
    ServiceServerFn,
};
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, quote};

impl ToTokens for ServerTrait<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            names: Names { server_trait, .. },
            docs,
            methods,
            generic_types,
            generic_bounds,
        } = self;
        tokens.append_all(quote! {
            #(#[doc = #docs])*
            ///
            /// This is the trait which is used by the server side in order to serve the client's
            /// requests
            pub trait #server_trait<_Format, #(#generic_types),*>: Send + Sync + 'static
            where
                _Format: imports::Format,
                #(#generic_bounds),*
            {
                #(#methods)*
            }
        });
    }
}

impl ToTokens for ServiceServerFn<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            name,
            docs,
            arg_names,
            arg_types,
            return_type,
        } = self;
        let arg_types = arg_types.iter().map(|p| &p.server_input);
        let return_type = &return_type.server_output;
        tokens.append_all(quote! {
            #(#[doc = #docs])*
            fn #name(&self, #(#arg_names: #arg_types),*) -> impl imports::Future<Output=#return_type> + Send;
        });
    }
}

impl ToTokens for ServerHandler<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            names: Names { server_handler, .. },
            generic_types,
            generic_bounds,
        } = self;
        let where_clause = GenericWhere(generic_bounds);
        tokens.append_all(quote!{
            #[doc = " A [Handler](imports::Handler) which handles requests/responses for a given service"]
            #[derive(Debug, Clone)]
            pub struct #server_handler<_Server, _Format, #(#generic_types,)*> #where_clause {
                server: _Server,
                format: _Format,
                _generics: imports::PhantomData<fn() -> (#(#generic_types,)*)>,
            }
        });
    }
}

impl ToTokens for ServerHandlerImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            names:
                Names {
                    server_handler,
                    server_trait,
                    ..
                },
            read_types,
            write_types,
            methods,
            generic_types,
            generic_bounds,
        } = self;
        tokens.append_all(quote! {
            impl<_Server, _Format, #(#generic_types),*> imports::Handler for #server_handler<_Server, _Format, #(#generic_types),*>
            where
                _Server: #server_trait<_Format, #(#generic_types),*>,
                #(_Format: imports::ReadFormat<#read_types>,)*
                #(_Format: imports::WriteFormat<#write_types>,)*
                #(#generic_bounds,)*
            {
                async fn handle(
                    &self,
                    request_id: imports::RequestID,
                    method_id: imports::MethodID,
                    request: Vec<u8>,
                    manager: imports::ServiceManager,
                    guard: imports::TeardownGuard,
                ) -> Result<Vec<u8>, imports::HandleError> {
                    let format = &self.format;
                    match method_id.value() {
                        #(#methods)*
                        _ => return Err(imports::HandleError::MethodNotFound)
                    }
                }
            }
        });
    }
}

impl ToTokens for HandlerMatchArm<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            name,
            id,
            arg_names,
            arg_types,
            return_type,
            prepare_input,
            prepare_output,
        } = self;
        let return_type = &return_type.transport;
        let arg_types = arg_types.iter().map(|p| &p.transport);
        tokens.append_all(quote! {
            #id => {
                let (#(#arg_names,)*): (#(#arg_types,)*) = self.format.read(&request).map_err(|error| {
                    imports::HandleError::BadPayloadFormat(error.to_string())
                })?;

                #(let #arg_names= {
                    let value = #arg_names;
                    #prepare_input
                };)*
                let value = self.server.#name(#(#arg_names),*).await;
                let value: #return_type = #prepare_output;
                self.format.write(value).map_err(|error| {
                    imports::HandleError::InternalError(format!("Error writing response: {error}"))
                })
            }
        });
    }
}
