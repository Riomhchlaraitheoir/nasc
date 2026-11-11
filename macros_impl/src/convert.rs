#![allow(
    clippy::too_many_lines,
    reason = "The functions are more simple as single function that breaking them up"
)]

use crate::input::Input;
use crate::output::{
    ClientImpl, ClientImplFn, ClientStruct, ClientTrait, ClientTraitFn, HandlerMatchArm, Names,
    Output, Param, PrepareInput, PrepareOutput, ServerHandler, ServerHandlerImpl, ServerTrait,
    Service, ServiceImpl, ServiceMethodIds, ServiceServerFn, ServiceServerImpl,
};
use crate::{ident_ccase, input};
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};

impl Names {
    fn new(service_name: &Ident) -> Self {
        Self {
            module: ident_ccase!(snake, service_name),
            service: service_name.clone(),
            server_trait: format_ident!("{}Server", service_name),
            server_handler: format_ident!("{}Handler", service_name),
            client_trait: format_ident!("{}Client", service_name),
            client_struct: format_ident!("{}ClientImpl", service_name),
        }
    }
}

impl<'a> Output<'a> {
    pub fn from(input: &'a Input) -> Self {
        let params: Vec<_> = input
            .methods
            .iter()
            .map(|m| {
                let args: Vec<_> = m.args.iter().map(|(_, p)| p.to_output()).collect();
                let ret = m.ret.to_output();
                (args, ret)
            })
            .collect();
        let client_to_server_types: Vec<TokenStream> = params
            .iter()
            .map(|(args, _)| {
                let args = args.iter().map(|p| p.transport.clone());
                quote!((#(#args,)*))
            })
            .chain(
                params
                    .iter()
                    .flat_map(|(args, _)| args.iter().flat_map(|p| p.stream_items.iter()))
                    .cloned(),
            )
            .collect();
        let server_to_client_types: Vec<TokenStream> = params
            .iter()
            .map(|(_, ret)| ret.transport.clone())
            .chain(
                params
                    .iter()
                    .flat_map(|(_, ret)| ret.stream_items.iter())
                    .cloned(),
            )
            .collect();

        let names = Names::new(&input.name);
        Self {
            names: names.clone(),
            crate_name: &input.args.nasc,
            vis: &input.vis,
            reexports: vec![
                names.service.clone(),
                names.client_trait.clone(),
                names.client_struct.clone(),
                names.server_handler.clone(),
                names.server_trait.clone(),
            ],
            service: Service {
                names: names.clone(),
                docs: &input.docs,
                generic_types: &input.generic_types,
                generic_bounds: &input.generic_bounds,
            },
            service_method_ids: ServiceMethodIds {
                names: names.clone(),
                method_names: input.methods.iter().map(|m| &m.name).collect(),
                method_ids: input.methods.iter().map(|m| &m.id).collect(),
                generic_types: &input.generic_types,
                generic_bounds: &input.generic_bounds,
            },
            service_impl: ServiceImpl {
                names: names.clone(),
                generic_types: &input.generic_types,
                generic_bounds: &input.generic_bounds,
            },
            service_server_impl: ServiceServerImpl {
                names: names.clone(),
                generic_types: &input.generic_types,
                generic_bounds: &input.generic_bounds,
            },
            server_trait: ServerTrait {
                names: names.clone(),
                docs: &input.docs,
                methods: input
                    .methods
                    .iter()
                    .zip(params.iter())
                    .map(|(m, (args, ret))| {
                        let arg_names = m.args.iter().map(|(a, _)| a).collect();
                        ServiceServerFn {
                            name: &m.name,
                            docs: &m.docs,
                            arg_names,
                            arg_types: args.clone(),
                            return_type: ret.clone(),
                        }
                    })
                    .collect(),
                generic_types: &input.generic_types,
                generic_bounds: &input.generic_bounds,
            },
            server_handler: ServerHandler {
                names: names.clone(),
                generic_types: &input.generic_types,
                generic_bounds: &input.generic_bounds,
            },
            server_handler_impl: ServerHandlerImpl {
                names: names.clone(),
                read_types: client_to_server_types.clone(),
                write_types: server_to_client_types.clone(),
                methods: input
                    .methods
                    .iter()
                    .zip(params.iter())
                    .map(|(m, (args, ret))| HandlerMatchArm {
                        name: &m.name,
                        id: &m.id,
                        arg_names: m.args.iter().map(|(n, _)| n).collect(),
                        arg_types: args.clone(),
                        return_type: ret.clone(),
                        prepare_input: m.args.iter().map(|(_, p)| PrepareInput(p)).collect(),
                        prepare_output: PrepareOutput(&m.ret),
                    })
                    .collect(),
                generic_types: &input.generic_types,
                generic_bounds: &input.generic_bounds,
            },
            client_trait: ClientTrait {
                names: names.clone(),
                methods: input
                    .methods
                    .iter()
                    .zip(params.iter())
                    .map(|(m, (args, ret))| ClientTraitFn {
                        name: &m.name,
                        docs: &m.docs,
                        arg_names: m.args.iter().map(|(n, _)| n).collect(),
                        arg_types: args.clone(),
                        return_type: ret.clone(),
                    })
                    .collect(),
                generic_types: &input.generic_types,
                generic_bounds: &input.generic_bounds,
            },
            client_struct: ClientStruct {
                names: names.clone(),
                generic_types: &input.generic_types,
                generic_bounds: &input.generic_bounds,
            },
            client_impl: ClientImpl {
                names,
                docs: &input.docs,
                write_types: client_to_server_types,
                read_types: server_to_client_types,
                methods: input
                    .methods
                    .iter()
                    .zip(params)
                    .map(|(m, (args, ret))| ClientImplFn {
                        name: &m.name,
                        id: &m.id,
                        docs: &m.docs,
                        arg_names: m.args.iter().map(|(n, _)| n).collect(),
                        arg_types: args,
                        return_type: ret,
                        prepare_input: PrepareInput(&m.ret),
                        prepare_output: m.args.iter().map(|(_, p)| PrepareOutput(p)).collect(),
                    })
                    .collect(),
                generic_types: &input.generic_types,
                generic_bounds: &input.generic_bounds,
            },
        }
    }
}

impl input::Param {
    fn to_output(&self) -> Param {
        match self {
            Self::Simple(ty) => Param {
                transport: ty.to_token_stream(),
                server_input: ty.to_token_stream(),
                server_output: ty.to_token_stream(),
                client_input: ty.to_token_stream(),
                client_output: ty.to_token_stream(),
                stream_items: vec![],
            },
            Self::Stream(item) => Param {
                transport: quote!(u32),
                server_input: quote!(impl imports::Stream<Item = #item> + Send + 'static),
                server_output: quote!(impl imports::Stream<Item = #item> + Send + 'static),
                client_input: quote!(impl imports::Stream<Item = #item> + Send + 'static),
                client_output: quote!(impl imports::Stream<Item=#item> + Send + 'static),
                stream_items: vec![item.to_token_stream()],
            },
            Self::Service(service) => Param {
                transport: quote!(u32),
                server_input: quote!(<#service as imports::Service>::Client<_Format>),
                server_output: quote!(impl imports::IntoHandler<#service, _Format>),
                client_input: quote!(impl imports::IntoHandler<#service, Self::Format>),
                client_output: quote!(<#service as imports::Service>::Client<Self::Format>),
                stream_items: vec![],
            },
            Self::Option(some) => {
                let some = some.to_output();
                let stream_items = some.stream_items;
                let some = [
                    some.transport,
                    some.server_input,
                    some.server_output,
                    some.client_input,
                    some.client_output,
                ]
                .map(|ty| quote!(Option<#ty>));

                let [
                    transport,
                    server_input,
                    server_output,
                    client_input,
                    client_output,
                ] = some;
                Param {
                    transport,
                    server_input,
                    server_output,
                    client_input,
                    client_output,
                    stream_items,
                }
            }
            Self::Result(ok, err) => {
                let ok = ok.to_output();
                let err = err.to_output();
                let mut stream_items = ok.stream_items;
                stream_items.extend(err.stream_items);
                let param = [
                    (ok.transport, err.transport),
                    (ok.server_input, err.server_input),
                    (ok.server_output, err.server_output),
                    (ok.client_input, err.client_input),
                    (ok.client_output, err.client_output),
                ]
                .map(|(ok, err)| quote!(Result<#ok, #err>));
                let [
                    transport,
                    server_input,
                    server_output,
                    client_input,
                    client_output,
                ] = param;
                Param {
                    transport,
                    server_input,
                    server_output,
                    client_input,
                    client_output,
                    stream_items,
                }
            }
            Self::Tuple(elements) => {
                let elements = elements.iter().map(Self::to_output).collect::<Vec<_>>();
                let stream_items = elements
                    .iter()
                    .flat_map(|e| e.stream_items.iter())
                    .cloned()
                    .collect();
                let param = [
                    elements.iter().map(|p| &p.transport).collect(),
                    elements.iter().map(|p| &p.server_input).collect(),
                    elements.iter().map(|p| &p.server_output).collect(),
                    elements.iter().map(|p| &p.client_input).collect(),
                    elements.iter().map(|p| &p.client_output).collect(),
                ]
                .map(|elements: Vec<_>| quote!((#(#elements,)*)));
                let [
                    transport,
                    server_input,
                    server_output,
                    client_input,
                    client_output,
                ] = param;
                Param {
                    transport,
                    server_input,
                    server_output,
                    client_input,
                    client_output,
                    stream_items,
                }
            }
        }
    }
}
