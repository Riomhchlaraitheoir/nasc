
#[doc(hidden)]
pub mod imports {
    pub use crate::{
        Handler, Rpc, RpcWithServer,
        connection::{
            ServiceManager, TeardownGuard,
            client::{Client, ClientError, ClientResponseFuture},
            message::{StreamID, RequestID, ServiceID, HandleError},
            MethodID
        },
        runtime::AsyncRuntime,
        format::{IsFormat, Format, ReadFormat, WriteFormat},
        panic_handler::PanicStrategy,
        __add_stream_bounds, method_id, __transport_type, __for_special_values,
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
        const fn _assert_is_rpc_service<T: Rpc>() {}
        _assert_is_rpc_service::<$service>();
        $crate::__macros::imports::paste! {
            <$service>::[<METHOD_ID_ $method_name>]
        }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __generate_rpc {
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
                user = ($($arg_type:tt)*),
            })*],
            return_type = ($($return_type:tt)*),
            streamed_in = [$($streamed_in:ty),*],
            streamed_out = [$($streamed_out:ty),*],
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
#[doc = " This is the [Rpc](Rpc) definition for this service" ]
pub struct $name<$($generics),*>(PhantomData<fn() -> ($($generics,)*)>) where ():Sized, $($where_bounds)*;


impl<$($generics),*> $name<$($generics),*> where (): Sized, $($generics: 'static,)* $($where_bounds)* {
    $(const [<METHOD_ID_ $method_name>]: MethodID = MethodID::_new($method_id);)*
}

impl<$($generics),*> Rpc for $name<$($generics),*> where (): Sized, $($generics: 'static,)* $($where_bounds)* {
    fn service_name() -> &'static str {
        stringify!($name)
    }
}

impl<$($generics),*> $name<$($generics),*> where ():Sized, $($generics: 'static,)* $($where_bounds)* {
    pub fn client<_Format, _Runtime: AsyncRuntime>(client: Client<_Runtime>, format: _Format) -> impl [<$name Client>]<$($generics)*>
    where
        $(_Format: WriteFormat<($(__transport_type!($($arg_type)*),)*)>,)*
        $(_Format: ReadFormat<__transport_type!($($return_type)*)>,)*
        $($(_Format: ReadFormat<$streamed_out>,)*)*
        $($(_Format: WriteFormat<$streamed_in>,)*)*
        $($($streamed_in: Sync,)*)*
        $($($streamed_out: Sync,)*)*
        $($generics: Send + 'static,)*
        $($where_bounds)*
    {
        [<$name Client Impl>] {
            client,
            format,
            _phantom: PhantomData::<fn() -> ($($generics,)*)>
        }
    }
}


impl<_Server: [<$name Server>]<$($generics),*>, $($generics,)* _Runtime: AsyncRuntime, _Format: IsFormat> RpcWithServer<_Server, _Runtime, _Format> for $name<$($generics),*>
where
    ():Sized,
    $($generics: Send + 'static,)*
    [<$name Handler>]<_Server, _Format, $($generics),*>: Handler<_Runtime>,
    _Runtime: AsyncRuntime,
    _Format: IsFormat,
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
pub trait [<$name Server>]<$($generics),*>: Send + Sync + 'static {
    $(
    $(#[doc = $method_docs])*
    fn $method_name(&self, $($arg_name: __add_stream_bounds!(( + Send), $($arg_type)*)),*) -> impl Future<Output=__add_stream_bounds!(( + Send + 'static), $($return_type)*)> + Send;
    )*
}

/// A [Handler](Handler) which handles requests/responses for a given service
#[derive(Debug, Clone)]
pub struct [<$name Handler>]<_Server, _Format $(,$generics)*>{
    server: _Server,
    format: _Format,
    _phantom: PhantomData<fn() -> ($($generics,)*)>
}

impl<_Runtime: AsyncRuntime, _Server: [<$name Server>]<$($generics,)*>, _Format, $($generics,)*> Handler<_Runtime> for [<$name Handler>]<_Server, _Format, $($generics),*>
where
    ():Sized,
    $(_Format: ReadFormat<($(__transport_type!($($arg_type)*),)*)>,)*
    $(_Format: WriteFormat<__transport_type!($($return_type)*)>,)*
    $($(_Format: ReadFormat<$streamed_in>,)*)*
    $($(_Format: WriteFormat<$streamed_out>,)*)*
    $($($streamed_in: Sync,)*)*
    $($($streamed_out: Sync,)*)*
    $($generics: Send + 'static,)*
    $($where_bounds)*
{
    fn handle(
        &self,
        request_id: RequestID,
        method_id: MethodID,
        request: Vec<u8>,
        manager: ServiceManager<_Runtime>,
        guard: TeardownGuard
    ) -> BoxFuture<Result<Vec<u8>, HandleError>> {
        Box::pin(async move {
        match method_id.value() {
            $(
            $method_id => {
                let ($($arg_name,)*): ($(__transport_type!($($arg_type)*),)*) = self.format.read(&request).map_err(|error| {
                    HandleError::BadPayloadFormat(error.to_string())
                })?;
                $(let $arg_name = __for_special_values!($arg_name, __incoming_values((manager: manager, format: self.format, guard: guard)), $($arg_type)*);)*
                let value = self.server.$method_name($($arg_name),*).await;
                let value = __for_special_values!(value, __outgoing_values((manager: manager, format: self.format, guard: guard)), $($return_type)*);
                self.format.write(value).map_err(|error| {
                    HandleError::InternalError(format!("Error writing response: {}", error))
                })
            }
            )*
            _ => return Err(HandleError::MethodNotFound),
        }
        })
    }
}

pub trait [<$name Client>]<$($generics),*>: Send + Sync where ():Sized, $($where_bounds)* {
    type Runtime: AsyncRuntime;
    $(
    $(#[doc = $method_docs])*
    ///
    /// Returns a future builder that allows certain options to be set before `await`-ing
    ///
    /// see [`ClientResponseFuture`]
    fn $method_name(&self, $($arg_name: __add_stream_bounds!(( + Send + 'static), $($arg_type)*)),*) -> ClientResponseFuture<impl Future<Output=Result<__add_stream_bounds!(( + Send), $($return_type)*), ClientError>> + Send, Self::Runtime>;
    )*
}

pub struct [<$name Client Impl>]<_Format, _Runtime: AsyncRuntime, $($generics),*> where ():Sized, $($where_bounds)* {
    client: Client<_Runtime>,
    format: _Format,
    _phantom: PhantomData<fn() -> ($($generics,)*)>
}

$(#[doc = $docs])*
///
/// This is the async client for the service, it produces requests from method calls
/// and sends the requests with the given [client](Client) before returning the response
///
///  The return value is always wrapped in a result: `Result<T, ClientError<_Conn::SendError>>` where `T` is the service return value
impl<_Format, _Runtime: AsyncRuntime, $($generics),*> [<$name Client>]<$($generics)*> for [<$name Client Impl>]<_Format, _Runtime, $($generics),*>
where
    $(_Format: WriteFormat<($(__transport_type!($($arg_type)*),)*)>,)*
    $(_Format: ReadFormat<__transport_type!($($return_type)*)>,)*
    $($(_Format: ReadFormat<$streamed_out>,)*)*
    $($(_Format: WriteFormat<$streamed_in>,)*)*
    $($($streamed_in: Sync,)*)*
    $($($streamed_out: Sync,)*)*
    $($generics: Send + 'static,)*
    $($where_bounds)* {
    type Runtime = _Runtime;

    $(
    $(#[doc = $method_docs])*
    fn $method_name(&self, $($arg_name: __add_stream_bounds!(( + Send + 'static), $($arg_type)*)),*) -> ClientResponseFuture<impl Future<Output=Result<__add_stream_bounds!(( + Send), $($return_type)*), ClientError>> + Send, Self::Runtime> {
        let request = {
            let format = self.format.clone();
            let manager = self.client.service_manager();
            let guard = self.client.guard.clone();
            async move {
                $(let $arg_name = __for_special_values!($arg_name, __outgoing_values((manager: manager, format: format, guard: guard)), $($arg_type)*);)*
                format.write(($($arg_name,)*)).map_err(ClientError::Serialise)
            }
        };
        let response = {
            let format = self.format.clone();
            let manager = self.client.service_manager();
            let guard = self.client.guard.clone();
            async move |response: Vec<u8>| -> Result<_, ClientError> {
                let value: __transport_type!($($return_type)*) = format.read(&response).map_err(ClientError::Deserialise)?;
                let value = __for_special_values!(value, __incoming_values((manager: manager, format: format, guard: guard)), $($return_type)*);
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
macro_rules! __outgoing_values {
    (stream<$item:ty>: $value:ident, (manager: $manager:expr, format: $format:expr, guard: $guard:expr)) => {{
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
}

#[doc(hidden)]
#[macro_export]
macro_rules! __incoming_values {
    (stream<$item:ty>: $value:ident, (manager: $manager:expr, format: $format:expr, guard: $guard:expr)) => {{
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
}

#[doc(hidden)]
#[macro_export]
macro_rules! __for_special_values {
    // tuple
    ($value:ident, $macro_name:ident($args:tt), ($($t:ty),*$(,)?)) => {
        $crate::__for_special_values!(tuple: $value, $macro_name($args), ($($t,)*))
    };
    (tuple: $value:ident, $macro_name:ident($args:tt), ($($t:ty,)*)) => {
        $crate::__for_special_values!(named_tuple: $value, $macro_name($args), a, ($($t,)*))
    };
    (named_tuple: $value:ident, $macro_name:ident($args:tt), $i:tt, ($(#$name:tt: $named:ty,)*$t:ty,$($unnamed:ty,)*)) => {
        $crate::__for_special_values!(named_tuple: $value, $macro_name($args), [<$i a>], ($(#$name: $named,)* #$i: $t, $($unnamed,)*))
    };
    (named_tuple: $value:ident, $macro_name:ident($args:tt), $_:tt, ($(#$name:tt: $t:ty,)*)) => {{
        let ($($name,)*) = $value;
        $(
        let $name = $crate::__for_special_values!($name, $macro_name($args), $t);
        )*
        ($($name,)*)
    }};
    ($value:ident, $macro_name:ident($args:tt), Result<$ok:ty, $err:ty>) => {
        match $value {
            Ok($value) => {
                Ok($crate::__for_special_values!($value, $macro_name($args), $ok))
            },
            Err($value) => {
                Err($crate::__for_special_values!($value, $macro_name($args), $err))
            },
        }
    };
    ($value:ident, $macro_name:ident($args:tt), Option<$ty:ty>) => {
        match $value {
            Some($value) => {
                Some($crate::__for_special_values!($value, $macro_name($args), $ty))
            },
            None => None
        }
    };
    ($value:ident, $macro_name:ident($args:tt), impl $($(::)?futures::stream::)?Stream<Item = $item:ty> $($additional_bounds:tt)*) => {
        $crate::$macro_name!(stream<$item>: $value, $args)
    };
    ($value:ident, $macro_name:ident($args:tt), impl Rpc<$rpc:ty>) => {
        $crate::$macro_name!(rpc<$rpc>: $value, $args)
    };
    ($value:ident, $macro_name:ident($args:tt), impl $($unexpected:tt)*) => {
        compile_error!(concat!("unexpected impl argument: impl ", stringify!($($unexpected)*)));
    };
    ($value:ident, $macro_name:ident($args:tt), $other:ty) => {
        $value
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __transport_type {
    // tuple
    (()) => {
        ()
    };
    (($t:ty)) => {
        $crate::__transport_type!($t)
    };
    (($($t:ty),*$(,)?)) => {
        $($crate::__transport_type!($t),)*
    };
    (Result<$ok:ty, $err:ty>) => {
        Result<$crate::__transport_type!($ok), $crate::__transport_type!($err)>
    };
    (Option<$ty:ty>) => {
        Option<$crate::__transport_type!($ty)>
    };
    (impl Stream<Item = $item:ty> $($additional_bounds:tt)*) => {
        u32
    };
    (impl futures::stream::Stream<Item = $item:ty> $($additional_bounds:tt)*) => {
        u32
    };
    (impl $($unexpected:tt)*) => {
        compile_error!(concat!("unexpected impl argument: impl ", stringify!($($unexpected)*)))
    };
    ($other:ty) => {
        $other
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __add_stream_bounds {
    // tuple
    ($bounds:tt, ($($t:ty),*$(,)?)) => {
        ($($crate::__add_stream_bounds!($bounds, $t),)*)
    };
    ($bounds:tt, Result<$ok:ty, $err:ty>) => {
        Result<$crate::__add_stream_bounds!($bounds, $ok), $crate::__add_stream_bounds!($bounds, $err)>
    };
    ($bounds:tt, Option<$ty:ty>) => {
        Option<$crate::__add_stream_bounds!($bounds, $ty)>
    };
    (($($bounds:tt)*),  impl futures::stream::Stream<Item = $item:ty> $($additional_bounds:tt)*) => {
        impl Stream<Item = $item> $($bounds)* $($additional_bounds:tt)*
    };
    ($bounds:tt, impl $($unexpected:tt)*) => {
        compile_error!(concat!("unexpected impl argument: impl ", stringify!($($unexpected)*)))
    };
    ($bounds:tt, $other:ty) => {
        $other
    };
}
