use crate::input::Param;
use crate::output::{PrepareInput, PrepareOutput};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Index, Member};

impl ToTokens for PrepareInput<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.0.prepare_input());
    }
}

impl ToTokens for PrepareOutput<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.0.prepare_output());
    }
}

impl Param {
    fn prepare_input(&self) -> TokenStream {
        match self {
            Self::Simple(_) => quote!(value),
            Self::Stream(stream) => quote!{
                {
                    let format = format.clone();
                    imports::StreamExt::filter_map(manager.incoming_stream(imports::StreamID::from(value), guard.clone()).await, move |bytes| {
                        let format = format.clone();
                        async move {
                            match imports::ReadFormat::<#stream>::read(&format, &bytes) {
                                Ok(value) => Some(value),
                                Err(error) => {
                                    imports::tracing::error!("Failed to deserialise bytes from stream: {error}");
                                    None
                                }
                            }
                        }
                    })
                }
            },
            Self::Service(service) => quote!{
                manager.incoming_service::<#service, _>(imports::ServiceID::from(value), guard.clone(), format.clone()).await
            },
            Self::Option(some) => {
                let some = some.prepare_input();
                quote! {
                    if let Some(value) = value {
                        Some(#some)
                    } else {
                        None
                    }
                }
            }
            Self::Result(ok, err) => {
                let ok = ok.prepare_input();
                let err = err.prepare_input();
                quote! {
                    match value {
                        Ok(value) => Ok(#ok),
                        Err(value) => Err(#err)
                    }
                }
            }
            Self::Tuple(elements) => {
                let i = (0..elements.len()).map(Index::from).map(Member::Unnamed);
                let elements = elements.iter().map(Self::prepare_input);
                quote! {
                    (#({
                        let value = value.#i;
                        #elements
                    },)*)
                }
            }
        }
    }
    fn prepare_output(&self) -> TokenStream {
        match self {
            Self::Simple(_) => quote!{value},
            Self::Stream(_) => quote!{
                {
                    let format = format.clone();
                    let stream = imports::StreamExt::filter_map(value, move |item| {
                        let format = format.clone();
                        async move {
                            match format.write(item) {
                                Ok(value) => Some(value),
                                Err(error) => {
                                    imports::tracing::error!("Failed to deserialise bytes from stream: {error}");
                                    None
                                }
                            }
                        }
                    });
                    let value: u32 = manager.outgoing_stream(stream, guard.clone()).await.into();
                    value
                }
            },
            Self::Service(_) => quote!{
                manager.outgoing_service(value.into_handler(format.clone()), guard.clone()).await.as_u32()
            },
            Self::Option(some) => {
                let some = some.prepare_output();
                quote! {
                    if let Some(value) = value {
                        Some(#some)
                    } else {
                        None
                    }
                }
            }
            Self::Result(ok, err) => {
                let ok = ok.prepare_output();
                let err = err.prepare_output();
                quote! {
                    match value {
                        Ok(value) => Ok(#ok),
                        Err(value) => Err(#err)
                    }
                }
            }
            Self::Tuple(elements) => {
                let i = (0..elements.len()).map(Index::from).map(Member::Unnamed);
                let elements = elements.iter().map(Self::prepare_output);
                quote! {
                    (#({
                        let value = value.#i;
                        #elements
                    },)*)
                }
            }
        }
    }
}