mod params;
mod service;
mod module;
mod server;
mod client;

use crate::input;
use proc_macro2::TokenStream;
use syn::{Expr, Ident, Path, Visibility, WherePredicate};

#[derive(Debug, Clone)]
pub struct Names {
    pub module: Ident,
    pub service: Ident,
    pub server_trait: Ident,
    pub server_handler: Ident,
    pub client_trait: Ident,
    pub client_struct: Ident,
}

pub struct Output<'a> {
    pub names: Names,
    pub crate_name: &'a Path,
    pub reexports: Vec<Ident>,
    pub vis: &'a Visibility,
    pub service: Service<'a>,
    pub service_method_ids: ServiceMethodIds<'a>,
    pub service_impl: ServiceImpl<'a>,
    pub service_server_impl: ServiceServerImpl<'a>,
    pub server_trait: ServerTrait<'a>,
    pub server_handler: ServerHandler<'a>,
    pub server_handler_impl: ServerHandlerImpl<'a>,
    pub client_trait: ClientTrait<'a>,
    pub client_struct: ClientStruct<'a>,
    pub client_impl: ClientImpl<'a>,
}

pub struct Service<'a> {
    pub names: Names,
    pub docs: &'a [Expr],
    pub generic_types: &'a [Ident],
    pub generic_bounds: &'a [WherePredicate],
}

pub struct ServiceMethodIds<'a> {
    pub names: Names,
    pub method_names: Vec<&'a Ident>,
    pub method_ids: Vec<&'a Expr>,
    pub generic_types: &'a [Ident],
    pub generic_bounds: &'a [WherePredicate],

}

pub struct ServiceImpl<'a> {
    pub names: Names,
    pub generic_types: &'a [Ident],
    pub generic_bounds: &'a [WherePredicate],
}

pub struct ServiceServerImpl<'a> {
    pub names: Names,
    pub generic_types: &'a [Ident],
    pub generic_bounds: &'a [WherePredicate],
}

pub struct ServerTrait<'a> {
    pub names: Names,
    pub docs: &'a [Expr],
    pub methods: Vec<ServiceServerFn<'a>>,
    pub generic_types: &'a [Ident],
    pub generic_bounds: &'a [WherePredicate],
}

pub struct ServiceServerFn<'a> {
    pub name: &'a Ident,
    pub docs: &'a [Expr],
    pub arg_names: Vec<&'a Ident>,
    pub arg_types: Vec<Param>,
    pub return_type: Param
}

pub struct ServerHandler<'a> {
    pub names: Names,
    pub generic_types: &'a [Ident],
    pub generic_bounds: &'a [WherePredicate],
}

pub struct ServerHandlerImpl<'a> {
    pub names: Names,
    pub read_types: Vec<TokenStream>,
    pub write_types: Vec<TokenStream>,
    pub methods: Vec<HandlerMatchArm<'a>>,
    pub generic_types: &'a [Ident],
    pub generic_bounds: &'a [WherePredicate],
}

pub struct HandlerMatchArm<'a> {
    pub name: &'a Ident,
    pub id: &'a Expr,
    pub arg_names: Vec<&'a Ident>,
    pub arg_types: Vec<Param>,
    pub return_type: Param,
    pub prepare_input: Vec<PrepareInput<'a>>,
    pub prepare_output: PrepareOutput<'a>,
}

pub struct ClientTrait<'a> {
    pub names: Names,
    pub methods: Vec<ClientTraitFn<'a>>,
    pub generic_types: &'a [Ident],
    pub generic_bounds: &'a [WherePredicate],
}

pub struct ClientTraitFn<'a> {
    pub name: &'a Ident,
    pub docs: &'a [Expr],
    pub arg_names: Vec<&'a Ident>,
    pub arg_types: Vec<Param>,
    pub return_type: Param
}

pub struct ClientStruct<'a> {
    pub names: Names,
    pub generic_types: &'a [Ident],
    pub generic_bounds: &'a [WherePredicate],
}

pub struct ClientImpl<'a> {
    pub names: Names,
    pub docs: &'a [Expr],
    pub write_types: Vec<TokenStream>,
    pub read_types: Vec<TokenStream>,
    pub methods: Vec<ClientImplFn<'a>>,
    pub generic_types: &'a [Ident],
    pub generic_bounds: &'a [WherePredicate],
}

pub struct ClientImplFn<'a> {
    pub name: &'a Ident,
    pub id: &'a Expr,
    pub docs: &'a [Expr],
    pub arg_names: Vec<&'a Ident>,
    pub arg_types: Vec<Param>,
    pub return_type: Param,
    pub prepare_input: PrepareInput<'a>,
    pub prepare_output: Vec<PrepareOutput<'a>>,
}

#[derive(Clone)]
pub struct Param {
    pub transport: TokenStream,
    pub server_input: TokenStream,
    pub server_output: TokenStream,
    pub client_input: TokenStream,
    pub client_output: TokenStream,
    pub stream_items: Vec<TokenStream>,
}

pub struct PrepareInput<'a>(pub &'a input::Param);
pub struct PrepareOutput<'a>(pub &'a input::Param);

struct GenericWhere<'a>(&'a [WherePredicate]);

#[macro_export]
macro_rules! ident_ccase {
    ($case:ident, $ident:expr) => {
        syn::Ident::new(&::convert_case::ccase!($case, $ident.to_string()), $ident.span())
    };
}