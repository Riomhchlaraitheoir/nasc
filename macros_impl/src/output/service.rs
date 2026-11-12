use crate::ident_ccase;
use crate::output::{
    GenericWhere, Names, Service, ServiceImpl, ServiceMethodIds, ServiceServerImpl,
};
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, format_ident, quote};

impl ToTokens for Service<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            names,
            docs,
            generic_types,
            generic_bounds,
        } = self;
        let service = &names.service;
        let where_clause = GenericWhere(generic_bounds);
        tokens.append_all(quote! {
            #(#[doc = #docs])*
            ///
            /// This is the [`Service`](imports::Service) definition for this service
            pub struct #service<#(#generic_types),*>(imports::PhantomData<fn() -> (#(#generic_types,)*)>) #where_clause;
        });
    }
}

impl ToTokens for ServiceMethodIds<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            names: Names { service, .. },
            method_names,
            method_ids,
            generic_types,
            generic_bounds,
        } = self;
        let where_clause = GenericWhere(generic_bounds);
        let method_names = method_names
            .iter()
            .map(|name| format_ident!("METHOD_ID_{}", ident_ccase!(upper_snake, name)));
        tokens.append_all(quote! {
            impl<#(#generic_types),*> #service<#(#generic_types),*> #where_clause {
                #(
                    pub const #method_names: imports::MethodID = imports::MethodID::_new(#method_ids);
                )*
            }
        });
    }
}

impl ToTokens for ServiceImpl<'_> {
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
        tokens.append_all(quote! {
            impl<#(#generic_types),*> imports::Service for #service<#(#generic_types),*> #where_clause {
                type Client<_Format: imports::Format> = #client_struct<_Format, #(#generic_types),*>;

                fn client<_Format: imports::Format>(client: imports::Client<Self>, format: _Format) -> Self::Client<_Format> {
                    #client_struct {
                        client: ::std::sync::Arc::new(client),
                        format,
                        _generics: imports::PhantomData,
                    }
                }

                fn service_name() -> &'static str {
                    stringify!(#service)
                }
            }
        });
    }
}

impl ToTokens for ServiceServerImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            names:
                Names {
                    service,
                    server_trait,
                    server_handler,
                    ..
                },
            generic_types,
            generic_bounds,
        } = self;
        tokens.append_all(quote! {
            impl<_Server, _Format, #(#generic_types),*> imports::ServiceWithServer<_Server, _Format> for #service<#(#generic_types),*>
            where
                _Format: imports::Format,
                #server_handler<_Server, _Format, #(#generic_types),*>: imports::Handler,
                _Server: #server_trait<_Format, #(#generic_types),*>,
                #(#generic_bounds,)*
            {
                type Handler = #server_handler<_Server, _Format, #(#generic_types),*>;

                fn handler(server: _Server, format: _Format) -> Self::Handler {
                    #server_handler{ server, format, _generics: imports::PhantomData }
                }
            }
        });
    }
}

impl ToTokens for GenericWhere<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.0.is_empty() {
            return;
        }
        let predicates = self.0;
        tokens.append_all(quote! {
            where #(#predicates),*
        });
    }
}
