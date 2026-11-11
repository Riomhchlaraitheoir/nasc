use crate::output::Output;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

impl ToTokens for Output<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            names,
            crate_name: crate_path,
            vis,
            reexports,
            service,
            service_method_ids,
            service_impl,
            service_server_impl,
            server_trait,
            server_handler,
            server_handler_impl,
            client_trait,
            client_struct,
            client_impl,
        } = self;
        let module = &names.module;

        let module = quote! {

            #[allow(
                unused_imports,
                reason = "These might not always be used, but they should be available in this module anyway"
            )]
            #vis use #module::{#(#reexports),*};

            #[allow(clippy::unused_unit)]
            #[allow(clippy::multiple_bound_locations)]
            #[allow(
                unused_imports,
                reason = "These might not always be used, but it's easier to include always"
            )]

            mod #module {
                use super::*;
                use #crate_path::__macros::{imports, traits::*};

                #service
                #service_method_ids
                #service_impl
                #service_server_impl
                #server_trait
                #server_handler
                #server_handler_impl
                #client_trait
                #client_struct
                #client_impl
            }

        };
        tokens.append_all(module);
    }
}
