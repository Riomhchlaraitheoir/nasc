use crate::{Param, Rpc};
use convert_case::ccase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{Field, FieldMutability, GenericParam, LitStr, Type, Visibility, parse_quote};

macro_rules! ident_ccase {
    ($case:ident, $ident:expr) => {
        Ident::new(&ccase!($case, $ident.to_string()), $ident.span())
    };
}

impl ToTokens for Rpc {
    #[allow(
        clippy::too_many_lines,
        reason = "This is a long function, but not too complex and splitting it would likely make it more confusing not less"
    )]
    fn to_token_stream(&self) -> TokenStream {
        let trait_rpc = &self.args.trait_rpc;
        let serde = &self.args.serde;
        let service = &self.name;
        let module = ident_ccase!(snake, service);

        let generics_with_bounds = &self.generics;
        let gen_params_with_bounds: Vec<_> = generics_with_bounds.params.iter().collect();
        let gen_params: Vec<_> = generics_with_bounds
            .params
            .iter()
            .map(|p| {
                if let GenericParam::Type(p) = p {
                    p.ident.to_token_stream()
                } else {
                    p.to_token_stream()
                }
            })
            .collect();
        let maybe_generics = if gen_params.is_empty() {
            vec![]
        } else {
            vec![gen_params.clone()]
        };
        let generics = quote!(#(<#(#maybe_generics),*>)*);

        let phantom_data = if generics_with_bounds.params.is_empty() {
            TokenStream::new()
        } else {
            quote!((PhantomData<fn() -> (#(#gen_params,)*)>))
        };
        let phantom_data_new = if generics_with_bounds.params.is_empty() {
            TokenStream::new()
        } else {
            quote!(PhantomData::<fn() -> (#(#gen_params,)*)>)
        };
        let docs = if self.docs.is_empty() {
            None
        } else {
            Some(&self.docs)
        }
        .into_iter()
        .collect::<Vec<_>>();
        let server = format_ident!("{}Server", service);
        let client = format_ident!("{}Client", service);
        let handler = format_ident!("{}Handler", service);

        let imports = {
            let vis = &self.vis;
            quote!(
                #vis use #module::{
                    #service,
                    #client,
                    #server
                };
            )
        };

        let request_variants: Vec<_> = self
            .methods
            .iter()
            .map(|method| {
                let snake_name = method.name.to_string();
                let name = ident_ccase!(pascal, method.name);
                let fields: Vec<_> = method
                    .args
                    .iter()
                    .map(|(_, param)| Field {
                        attrs: vec![],
                        vis: Visibility::Inherited,
                        mutability: FieldMutability::None,
                        ident: None,
                        colon_token: None,
                        ty: param.for_transport(),
                    })
                    .collect();
                quote!(
                    #[serde(rename = #snake_name)]
                    #name(#(#fields),*)
                )
            })
            .collect();

        let response_variants = self.methods.iter().map(|method| {
            let snake_name = method.name.to_string();
            let name = ident_ccase!(pascal, method.name);
            let ret = method.ret.for_transport();
            quote!(
                #[serde(rename = #snake_name)]
                #name(#ret)
            )
        });
        let request_to_name = self.methods.iter().map(|method| {
            let name = method.name.to_string();
            let variant = ident_ccase!(pascal, method.name);
            quote!(Self::#variant(..) => Cow::Borrowed(#name))
        });
        let response_to_name = self.methods.iter().map(|method| {
            let name = method.name.to_string();
            let variant = ident_ccase!(pascal, method.name);
            quote!(Self::#variant(..) => #name)
        });

        let server_fns = self.methods.iter().map(|method| {
            let name = &method.name;
            let params = method.args.iter().map(|(pat, param)| {
                let param = param.normal(&TokenStream::new());
                quote!(#pat: #param)
            });
            let docs = &method.docs;
            let docs = quote! {
                #(#[doc = #docs])*
            };
            let ret = method.ret.normal(&quote!( + MaybeSend + MaybeSync));
            quote! {
                #docs
                fn #name(&self #(,#params)*) -> impl Future<Output=#ret> + Send;
            }
        });
        let handle_arms: Vec<_> = self.methods.iter().map(|method| {
            let name = &method.name;
            let variant = ident_ccase!(pascal, method.name);
            let params = method.args.iter().map(|(pat, _)| pat).collect::<Vec<_>>();
            let receiving_streams = method.args.iter().map(|(pat, param)| {
                param.receiving_streams(&quote!(controller), pat)
            });
            let sending_streams = method.ret.sending_streams(&quote!(controller), &format_ident!("value"));
            quote! {
                Request::#variant(#(#params),*) => {
                    #(#receiving_streams)*
                    let value = self.0.#name(#(#params),*).await;
                    #sending_streams
                    Response::#variant(value)
                },
            }
        }).collect();

        let async_client_fns = self.client_fns();

        let trait_rpc_str = trait_rpc.to_token_stream().to_string().replace(' ', "");
        let async_client_docs = [
            LitStr::new(" This is the async client for the service, it produces requests from method calls", Span::call_site()),
            LitStr::new(" (including chained method calls) and sends the requests with the given", Span::call_site()),
            LitStr::new(&format!(" [transport]({trait_rpc_str}::Client) before returning the response"), Span::call_site()),
            LitStr::new("", Span::call_site()),
            LitStr::new(" The return value is always wrapped in a result: `Result<T, ClientError<_Conn::SendError>>` where `T` is the service return value", Span::call_site()),
        ].into_iter();

        let service_doc =
            format!(" This is the [Rpc]({trait_rpc_str}::Rpc) definition for this service");

        let streamed_from_client: Vec<_> = self
            .methods
            .iter()
            .flat_map(|method| {
                method
                    .args
                    .iter()
                    .flat_map(|(_, param)| param.streamed_types())
            })
            .collect();
        let streamed_from_server: Vec<_> = self
            .methods
            .iter()
            .flat_map(|method| method.ret.streamed_types())
            .collect();
        let client_stream_format_bounds = {
            let mut bounds = TokenStream::new();
            for ty in &streamed_from_client {
                bounds.extend(quote!( + WriteFormat<#ty>));
            }
            for ty in &streamed_from_server {
                bounds.extend(quote!( + ReadFormat<#ty>));
            }
            bounds
        };
        let server_stream_format_bounds = streamed_from_server
            .iter()
            .map(|ty| quote!(WriteFormat<#ty>))
            .chain(
                streamed_from_client
                    .iter()
                    .map(|ty| quote!(ReadFormat<#ty>)),
            );
        let streamed_values: Vec<_> = streamed_from_client
            .iter()
            .chain(streamed_from_server.iter())
            .cloned()
            .collect();

        quote! {
            #[allow(unused_imports, reason = "These might not always be used, but they should be available in this module anyway")]
            #imports

            #[allow(unused_imports, reason = "These might not always be used, but it's easier to include always")]
            mod #module {
                use #trait_rpc::__macro_imports::*;
                use super::*;

                #(
                    #(#[doc = #docs])*
                    ///
                )*
                #[doc = #service_doc]
                pub struct #service #generics_with_bounds #phantom_data;

                impl #generics_with_bounds Rpc for #service <#(#gen_params),*> #(where #(#maybe_generics: Send + 'static),*)* {
                    type Client<_Conn: Connection, _Format> = #client<_Conn, _Format #(,#gen_params)*>;
                    type Request = Request #generics;
                    type Response = Response #generics;
                    fn service_name() -> &'static str {
                        stringify!(#service)
                    }
                    fn client<_Conn: Connection, _Format>(client: Client<_Conn, _Format>) -> Self::Client<_Conn, _Format> {
                        #client(client, #phantom_data_new)
                    }
                }

                impl<Server: #server #generics #(, #gen_params_with_bounds)*> RpcWithServer<Server> for #service #generics where #(#gen_params: Send + 'static),* {
                    type Handler<_Format> = #handler<Server #(, #gen_params)*>;
                    fn handler<_Format>(server: Server) -> Self::Handler<_Format> {
                        #handler(server, #phantom_data_new)
                    }
                }


                #[derive(Debug, Serialize, Deserialize)]
                #[serde(crate = #serde)]
                #[serde(tag = "method", content = "args")]
                pub enum Request #generics {
                    #(#request_variants,)*
                }

                impl #generics #trait_rpc::Request for Request #generics {
                    fn name(&self) -> Cow<'static, str> {
                        match self {
                            #(#request_to_name),*
                        }
                    }
                }

                #[derive(Debug, Serialize, Deserialize)]
                #[serde(crate = #serde)]
                #[serde(tag = "method", content = "result")]
                pub enum Response #generics {
                    #(#response_variants,)*
                }

                impl #generics Response #generics {
                    fn fn_name(&self) -> &'static str {
                        match self {
                            #(#response_to_name),*
                        }
                    }
                }

                #(
                    #(#[doc = #docs])*
                    ///
                )*
                /// This is the trait which is used by the server side in order to serve the client
                pub trait #server #generics: Send + Sync {
                    #(#server_fns)*
                }

                /// A [Handler](Handler) which handles requests/responses for a given service
                #[derive(Debug, Clone)]
                pub struct #handler<_Server #(,#gen_params)*>(_Server, #phantom_data);
                impl<_Server: #server #generics, _Format #(, #gen_params_with_bounds)*> Handler<_Format> for #handler<_Server #(,#gen_params)*>
                where
                    #(#gen_params: Send + 'static,)*
                    #(_Format: #server_stream_format_bounds,)*
                    #(#streamed_values: MaybeSync,)*
                {
                    type Rpc = #service #generics;
                    type Server = _Server;

                    fn server(&self) -> &Self::Server {
                        &self.0
                    }

                    async fn handle<'a>(&'a self, request: Request #generics, controller: &impl ConnectionController<'a, _Format>) -> Response #generics {
                        match request {
                            #(#handle_arms)*
                        }
                    }
                }

                #(
                    #(#[doc = #docs])*
                    ///
                )*
                #(#[doc = #async_client_docs])*
                #[derive(Clone)]
                pub struct #client<_Conn, _Format #(,#gen_params_with_bounds)*>(Client<_Conn, _Format>, #phantom_data);
                #[allow(clippy::future_not_send)]
                impl<_Conn, _Format #(, #gen_params_with_bounds)*> #client<_Conn, _Format #(,#gen_params)*>
                where
                    _Conn: Connection + MaybeSend + 'static,
                    _Format: Format<Response #generics, Request #generics> #client_stream_format_bounds,
                    #(#streamed_values: MaybeSend + MaybeSync,)*
                {
                    #(#async_client_fns)*
                }

            }
        }
    }

    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.to_token_stream());
    }
}

impl Rpc {
    fn client_fns(&self) -> impl Iterator<Item = TokenStream> {
        self.methods.iter().map(move |method| {
            let name = &method.name;
            let name_str = name.to_string();
            let params = method.args.iter().map(|(pat, param)| {
                let param = param.normal(&TokenStream::new());
                quote! { #pat: #param }
            });
            let args = method.args.iter().map(|(pat, _)| pat);
            let variant = ident_ccase!(pascal, name);
            let docs = &method.docs;
            let docs = quote! {
                #(#[doc = #docs])*
            };
            let ret = method.ret.normal(&TokenStream::new());
            let sending_streams = method.args.iter().map(|(pat, param)| {
                param.sending_streams(&quote!(self.0), pat)
            });
            let receiving_streams = method.ret.receiving_streams(&quote!(self.0), &format_ident!("value"));
            quote! {
                #docs
                pub async fn #name(&self #(, #params)*) -> Result<#ret, ClientError<_Conn::SendError>> {
                    #(#sending_streams)*
                    match self.0.send(Request::#variant(#(#args),*)).await? {
                        Response::#variant(value) => {
                            #receiving_streams
                            Ok(value)
                        },
                        other => Err(WrongResponseType::new(#name_str, other.fn_name()).into()),
                    }
                }
            }
        })
    }
}

impl Param {
    fn normal(&self, bounds: &TokenStream) -> TokenStream {
        match self {
            Self::Simple(ty) => ty.to_token_stream(),
            Self::Stream(ty) => {
                quote! { impl futures::stream::Stream<Item=#ty> #bounds }
            }
            Self::Option(param) => {
                let param = param.normal(bounds);
                quote! { Option<#param> }
            }
            Self::Result(ok, err) => {
                let ok = ok.normal(bounds);
                let err = err.normal(bounds);
                quote! { Result<#ok, #err> }
            }
            Self::Tuple(params) => {
                let params = params.iter().map(|p| p.normal(bounds));
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

    fn sending_streams(&self, controller: &TokenStream, value: &Ident) -> TokenStream {
        self.streams_inner(controller, value, &format_ident!("sending_stream"))
    }

    fn receiving_streams(&self, controller: &TokenStream, value: &Ident) -> TokenStream {
        self.streams_inner(controller, value, &format_ident!("receiving_stream"))
    }

    fn streams_inner(
        &self,
        controller: &TokenStream,
        value: &Ident,
        method: &Ident,
    ) -> TokenStream {
        if self.streamed_types().is_empty() {
            return TokenStream::new();
        }

        match self {
            Self::Simple(_) => TokenStream::new(),
            Self::Stream(ty) => {
                quote! {
                    let #value = #controller.#method::<#ty>(#value).await;
                }
            }
            Self::Option(param) => {
                let param = param.streams_inner(controller, value, method);
                quote! {
                    let #value = match #value {
                        Some(#value) => {
                            #param
                            Some(#value)
                        },
                        None => None,
                    };
                }
            }
            Self::Result(ok, err) => {
                let ok = ok.streams_inner(controller, value, method);
                let err = err.streams_inner(controller, value, method);
                quote! {
                    let #value = match #value {
                        Ok(#value) => {
                            #ok
                            Some(#value)
                        },
                        Err(#value) => {
                            #err
                            Some(#value)
                        },
                    };
                }
            }
            Self::Tuple(params) => {
                let (pats, params): (Vec<_>, Vec<_>) = params
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let value = format_ident!("{}_{}", value, i);
                        let param = p.streams_inner(controller, &value, method);
                        (value, param)
                    })
                    .unzip();
                quote! {
                    let (#(#pats,)*) = #value;
                    #(#params)*
                    let #value = (#(#pats,)*);
                }
            }
        }
    }
}
