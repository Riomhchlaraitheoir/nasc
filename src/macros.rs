
#[doc(hidden)]
pub mod imports {
    pub use crate::{
        Handler, Service, ServiceWithServer, IntoHandler,
        connection::{
            ServiceManager, TeardownGuard,
            client::{Client, ClientError, ClientResponseFuture},
            message::{StreamID, RequestID, ServiceID, HandleError},
            MethodID
        },
        format::{Format, ReadFormat, WriteFormat},
        method_id, __for_special_values, __for_special_types
    };
    pub use ::{
        futures::{
            sink::{Sink, SinkExt},
            stream::{Stream, StreamExt},
            future::BoxFuture
        },
        std::marker::PhantomData,
        tracing,
        pastey::paste
    };
}

#[macro_export]
macro_rules! method_id {
    ($service:ty => $method_name:ident) => {const {
        const fn _assert_is_service<T: Service>() {}
        _assert_is_service::<$service>();
        $crate::__macros::imports::paste! {
            <$service>::[<METHOD_ID_ $method_name>]
        }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __generate_service {
    (
        docs = { $($docs:expr),* },
        vis = $vis:vis,
        generics = [$($generics:ident),*],
        where_bounds = [$($where_bounds:tt)*],
        name = $name:ident,
        methods = [$({
            id = $method_id:expr,
            docs = { $($method_docs:expr),* },
            name = $method_name:ident,
            args = [$({
                name = $arg_name:ident,
                user = $arg_type:tt,
            })*],
            return_type = $return_type:tt,
            streamed_in = [$($streamed_in:ty),*],
            streamed_out = [$($streamed_out:ty),*],
            services_in = [$($services_in:ty),*],
            services_out = [$($services_out:ty),*],
        })*],
    ) => {
$crate::__macros::imports::paste!{
#[allow(unused_imports, reason = "These might not always be used, but they should be available in this module anyway")]
$vis use [<$name Module>]::{
    [<$name>],
    [<$name Client>],
    [<$name Server>]
};

#[allow(non_camel_case_types)]
#[allow(clippy::unused_unit)]
#[allow(clippy::multiple_bound_locations)]
#[allow(unused_imports, reason = "These might not always be used, but it's easier to include always")]
mod [<$name Module>] {

use super::*;
use $crate::__macros::imports::*;

$(#[doc = $docs])*
///
#[doc = " This is the [Service](Service) definition for this service" ]
pub struct $name<$($generics),*>(PhantomData<fn() -> ($($generics,)*)>) where ():Sized, $($where_bounds)*;


impl<$($generics),*> $name<$($generics),*> where (): Sized, $($generics: 'static,)* $($where_bounds)* {
    $(const [<METHOD_ID_ $method_name>]: MethodID = MethodID::_new($method_id);)*
}

impl<$($generics),*> $crate::Service for $name<$($generics),*> where (): Sized, $($generics: 'static,)* $($where_bounds)* {
    type Client<_Format: Format> = [<$name ClientImpl>]<_Format, $($generics),*>;

    fn client<_Format: Format>(client: Client<Self>, format: _Format) -> Self::Client<_Format> {
        [<$name Client Impl>] {
            client,
            format,
            _phantom: PhantomData::<fn() -> ($($generics,)*)>
        }
    }

    fn service_name() -> &'static str {
        stringify!($name)
    }
}

impl<_Server: [<$name Server>]<_Format, $($generics),*>, $($generics,)* _Format: Format> ServiceWithServer<_Server, _Format> for $name<$($generics),*>
where
    ():Sized,
    $($generics: Send + 'static,)*
    [<$name Handler>]<_Server, _Format, $($generics),*>: Handler,
    _Format: Format,
    $($where_bounds)*
{
    type Handler = [<$name Handler>]<_Server, _Format, $($generics),*>;

    fn handler(server: _Server, format: _Format) ->  Self::Handler {
        [<$name Handler>]{server, format, _phantom: PhantomData::<fn() -> ($($generics,)*)>}
    }
}

$(#[doc = $docs])*
///
/// This is the trait which is used by the server side in order to serve the client
pub trait [<$name Server>]<_Format: Format, $($generics),*>: Send + Sync + 'static
where
    (): Sized,
    $($where_bounds)*
{
    $(
    $(#[doc = $method_docs])*
    fn $method_name(&self, $($arg_name: __for_special_types!(server_input_type(format: _Format), $arg_type)),*) -> impl Future<Output=__for_special_types!(server_return_type(format: _Format), $return_type)> + Send;
    )*
}

/// A [Handler](Handler) which handles requests/responses for a given service
#[derive(Debug, Clone)]
pub struct [<$name Handler>]<_Server, _Format $(,$generics)*>{
    server: _Server,
    format: _Format,
    _phantom: PhantomData<fn() -> ($($generics,)*)>
}

impl<_Server, _Format, $($generics,)*> Handler for [<$name Handler>]<_Server, _Format, $($generics),*>
where
    ():Sized,
    _Server: [<$name Server>]<_Format, $($generics,)*>,
    $(_Format: ReadFormat<($(__for_special_types!(to_transport_type(), $arg_type),)*)>,)*
    $(_Format: WriteFormat<__for_special_types!(to_transport_type(), $return_type)>,)*
    $($(_Format: ReadFormat<$streamed_in>,)*)*
    $($(_Format: WriteFormat<$streamed_out>,)*)*
    $($($streamed_in: Sync,)*)*
    $($($streamed_out: Sync,)*)*
    $($generics: Send + 'static,)*
    $($where_bounds)*
{
    async fn handle(
        &self,
        request_id: RequestID,
        method_id: MethodID,
        request: Vec<u8>,
        manager: ServiceManager,
        guard: TeardownGuard
    ) -> Result<Vec<u8>, HandleError> {
        match method_id.value() {
            $(
            $method_id => {
                let ($($arg_name,)*): ($(__for_special_types!(to_transport_type(), $arg_type),)*) = self.format.read(&request).map_err(|error| {
                    HandleError::BadPayloadFormat(error.to_string())
                })?;
                $(let $arg_name = __for_special_values!($arg_name, incoming_values(manager: manager, format: self.format, guard: guard), $arg_type);)*
                let value = self.server.$method_name($($arg_name),*).await;
                let value = __for_special_values!(value, outgoing_values(manager: manager, format: self.format, guard: guard), $return_type);
                self.format.write(value).map_err(|error| {
                    HandleError::InternalError(format!("Error writing response: {}", error))
                })
            }
            )*
            _ => return Err(HandleError::MethodNotFound),
        }
    }
}

pub trait [<$name Client>]<$($generics),*>: Send + Sync where ():Sized, $($where_bounds)* {
    type Format: Format;
    $(
    $(#[doc = $method_docs])*
    ///
    /// Returns a future builder that allows certain options to be set before `await`-ing
    ///
    /// see [`ClientResponseFuture`]
    fn $method_name(&self, $($arg_name: __for_special_types!(client_input_type(format: Self::Format), $arg_type)),*) -> ClientResponseFuture<impl Future<Output=Result<__for_special_types!(client_return_type(format: Self::Format), $return_type), ClientError>> + Send>;
    )*
}

pub struct [<$name Client Impl>]<_Format, $($generics: 'static),*> where ():Sized, $($where_bounds)* {
    client: Client<$name<$($generics),*>>,
    format: _Format,
    _phantom: PhantomData<fn() -> ($($generics,)*)>
}

$(#[doc = $docs])*
///
/// This is the async client for the service, it produces requests from method calls
/// and sends the requests with the given [client](Client) before returning the response
///
///  The return value is always wrapped in a result: `Result<T, ClientError<_Conn::SendError>>` where `T` is the service return value
impl<_Format, $($generics),*> [<$name Client>]<$($generics)*> for [<$name Client Impl>]<_Format, $($generics),*>
where
    $(_Format: WriteFormat<($(__for_special_types!(to_transport_type(), $arg_type),)*)>,)*
    $(_Format: ReadFormat<__for_special_types!(to_transport_type(), $return_type)>,)*
    $($(_Format: ReadFormat<$streamed_out>,)*)*
    $($(_Format: WriteFormat<$streamed_in>,)*)*
    $($($streamed_in: Sync,)*)*
    $($($streamed_out: Sync,)*)*
    $($generics: Send + 'static,)*
    $($where_bounds)* {
    type Format = _Format;

    $(
    $(#[doc = $method_docs])*
    fn $method_name(&self, $($arg_name: __for_special_types!(client_input_type(format: _Format), $arg_type)),*) -> ClientResponseFuture<impl Future<Output=Result<__for_special_types!(client_return_type(format: _Format), $return_type), ClientError>> + Send> {
        let request = {
            let format = self.format.clone();
            let manager = self.client.service_manager();
            let guard = self.client.guard.clone();
            async move {
                $(let $arg_name = __for_special_values!($arg_name, outgoing_values(manager: manager, format: format, guard: guard), $arg_type);)*
                format.write(($($arg_name,)*)).map_err(ClientError::Serialise)
            }
        };
        let response = {
            let format = self.format.clone();
            let manager = self.client.service_manager();
            let guard = self.client.guard.clone();
            async move |response: Vec<u8>| -> Result<_, ClientError> {
                let value: __for_special_types!(to_transport_type(), $return_type) = format.read(&response).map_err(ClientError::Deserialise)?;
                let value = __for_special_values!(value, incoming_values(manager: manager, format: format, guard: guard), $return_type);
                Ok(value)
            }
        };
        self.client.send(request, MethodID::_new($method_id), response)
    }
    )*
}

}
}
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __for_special_types {
    // tuple
    ($mode:ident $args:tt, (normal: $ty:ty)) => {
        $ty
    };
    ($mode:ident $args:tt, (tuple: $($name:ident: $ty:tt),*)) => {
        ($($crate::__for_special_types!($mode $args, $ty)),*)
    };
    ($mode:ident $args:tt, (result: $ok:tt, $err:tt)) => {
        Result<$crate::__for_special_types!($mode $args, $ok), $crate::__for_special_types!($mode $args, $err)>
    };
    ($mode:ident $args:tt, (option: $ty:tt)) => {
        Option<$crate::__for_special_types!($mode $args, $ty)>
    };
    (to_transport_type(), (stream: $item:ty)) => {
        u32
    };
    (to_transport_type(), (service: $service:ty)) => {
        u32
    };
    (client_input_type(format: $format:ty), (stream: $item:ty)) => {
        impl Stream<Item = $item> + Send + 'static
    };
    (client_input_type(format: $format:ty), (service: $service:ty)) => {
        impl IntoHandler<$service, $format>
    };
    (client_return_type(format: $format:ty), (stream: $item:ty)) => {
        impl Stream<Item=$item>
    };
    (client_return_type(format: $format:ty), (service: $service:ty)) => {
        <$service as Service>::Client<$format>
    };
    (server_return_type(format: $format:ty), (stream: $item:ty)) => {
        impl Stream<Item = $item> + Send + 'static
    };
    (server_return_type(format: $format:ty), (service: $service:ty)) => {
        impl IntoHandler<$service, $format>
    };
    (server_input_type(format: $format:ty), (stream: $item:ty)) => {
        impl Stream<Item = $item> + Send + 'static
    };
    (server_input_type(format: $format:ty), (service: $service:ty)) => {
        <$service as Service>::Client<$format>
    };
    ($mode:ident: $($t:tt)*) => {
        compile_error!(concat!("Internal Error!: unexpected __for_special_types!(", stringify!($mode: $($t)*), ")"))
    };
    ($mode:ident $args:tt, impl $($unexpected:tt)*) => {
        compile_error!(concat!("unexpected impl argument: impl ", stringify!($($unexpected)*)))
    };
    ($mode:ident $args:tt, $other:ty) => {
        $other
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __for_special_values {
    // tuple
    ($value:ident, $mode:ident $args:tt, (tuple: $($name:ident: $ty:tt),*)) => {{
        let ($($name,)*) = $value;
        $(
        let $name = $crate::__for_special_values!($name, $mode $args, $ty);
        )*
        ($($name,)*)
    }};
    ($value:ident, $mode:ident $args:tt, (result: $ok:tt, $err:tt)) => {
        match $value {
            Ok($value) => {
                Ok($crate::__for_special_values!($value, $mode $args, $ok))
            },
            Err($value) => {
                Err($crate::__for_special_values!($value, $mode $args, $err))
            },
        }
    };
    ($value:ident, $mode:ident $args:tt, (option: $ty:tt)) => {
        match $value {
            Some($value) => {
                Some($crate::__for_special_values!($value, $mode $args, $ty))
            },
            None => None
        }
    };
    ($value:ident, outgoing_values(manager: $manager:expr, format: $format:expr, guard: $guard:expr), (stream: $item:ty)) => {{
        let format = $format.clone();
        use $crate::__macros::imports::StreamExt;
        let $value = StreamExt::filter_map($value, move |item|{
            let format = format.clone();
            async move {
                match format.write(item) {
                    Ok(value) => Some(value),
                    Err(error) => {
                        $crate::__macros::imports::tracing::error!("Failed to serialise item for stream: {error}");
                        None
                    }
                }
            }
        });
        let id: u32 = $manager.outgoing_stream($value, $guard.clone()).await.into();
        id
    }};
    ($value:ident, outgoing_values(manager: $manager:expr, format: $format:expr, guard: $guard:expr), (service: $service:ty)) => {{
        let $value = $value.into_handler($format.clone());
        let id: u32 = $manager.outgoing_service($value, $guard.clone()).await.into();
        id
    }};
    ($value:ident, incoming_values(manager: $manager:expr, format: $format:expr, guard: $guard:expr), (stream: $item:ty)) => {{
        let format = $format.clone();
        use $crate::__macros::imports::StreamExt;
        StreamExt::filter_map($manager.incoming_stream(StreamID::from($value), $guard.clone()).await, move |bytes| {
            let format = format.clone();
            async move {
                match ReadFormat::<$item>::read(&format, &bytes) {
                    Ok(value) => Some(value),
                    Err(error) => {
                        $crate::__macros::imports::tracing::error!("Failed to deserialise bytes from stream: {error}");
                        None
                    }
                }
            }
        })
    }};
    ($value:ident, incoming_values(manager: $manager:expr, format: $format:expr, guard: $guard:expr), (service: $service:ty)) => {{
        $manager.incoming_service::<$service, _>(ServiceID::from($value), $guard.clone(), $format.clone()).await
    }};
    ($value:ident, $mode:ident $args:tt, (normal: $ty:ty)) => {
        $value
    };
    ($($token:tt)*) => {
        compile_error!(concat!("unexpected input: ", stringify!($($token)*)))
    }
}
