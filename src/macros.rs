
#[doc(hidden)]
pub mod imports {
    pub use crate::{
        ConnectionController, Handler, MaybeSend, MaybeSync, Rpc, RpcWithServer, StreamError,
        client::{Client, ClientError, Connection, WrongResponseType},
        format::{Format, ReadFormat, WriteFormat},
        message::StreamID,
        Request as RpcRequest,
        __streams, __add_stream_bounds
    };
    pub use ::{
        futures::sink::{Sink, SinkExt},
        futures::stream::{Stream, StreamExt},
        serde::{Deserialize, Serialize},
        std::borrow::Cow,
        std::convert::Infallible,
        std::marker::PhantomData,
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __generate_rpc {
    (
        args = {
            serde = $serde:literal,
        },
        docs = { $($docs:expr),* },
        vis = $vis:vis,
        generics = [$($generics:ident),*],
        where_bounds = [$($where_bounds:tt)*],
        name = $name:ident,
        methods = [$({
            docs = { $($method_docs:expr),* },
            name = $method_name:ident,
            args = [$({
                name = $arg_name:ident,
                user = ($($arg_user_type:tt)*),
                transport = $arg_transport_type:ty,
            })*],
            return_type = {
                user = ($($user_return_type:tt)*),
                transport = $transport_return_type:ty,
            },
            streamed_in = [$($streamed_in:ty),*],
            streamed_out = [$($streamed_out:ty),*],
        })*],
    ) => {
$crate::__paste!{
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


impl<$($generics),*> Rpc for $name<$($generics),*> where (): Sized, $($generics: 'static,)* $($where_bounds)* {
    type Request = Request<$($generics),*>;
    type Response = Response<$($generics),*>;
    fn service_name() -> &'static str {
        stringify!($name)
    }
}

impl<$($generics),*> $name<$($generics),*> where ():Sized, $($generics: 'static,)* $($where_bounds)* {
    pub fn client<_Conn: Connection, _Format>(client: Client<_Conn, _Format>) -> impl [<$name Client>]<$($generics)*>
    where
        _Conn: Connection + MaybeSend + 'static,
        _Format: Format<Response<$($generics),*>, Request<$($generics),*>>,
        $($(_Format: ReadFormat<$streamed_out>,)*)*
        $($(_Format: WriteFormat<$streamed_in>,)*)*
        $($($streamed_in: MaybeSync,)*)*
        $($($streamed_out: MaybeSync,)*)*
        $($generics: Send + 'static,)*
        $($where_bounds)*
    {
        [<$name Client Impl>](client, PhantomData::<fn() -> ($($generics,)*)>)
    }
}


impl<_Server: [<$name Server>]<$($generics),*>, $($generics),*> RpcWithServer<_Server> for $name<$($generics),*> where ():Sized,  $($generics: Send + 'static,)* $($where_bounds)* {
    type Handler<_Format> = [<$name Handler>]<_Server, $($generics),*>;

    fn handler<_Format>(server: _Server) ->  Self::Handler<_Format> {
        [<$name Handler>](server, PhantomData::<fn() -> ($($generics,)*)>)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = $serde)]
#[serde(tag = "method", content = "args")]
pub enum Request<$($generics),*> {
    $(
    $(#[doc = $method_docs])*
    $method_name { $($arg_name: $arg_transport_type),* },
    )*
}

impl<$($generics),*> RpcRequest for Request<$($generics),*> {
    fn name(&self) -> &'static str {
        match self {
            $(Request::$method_name{..} => stringify!($method_name), )*
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = $serde)]
#[serde(tag = "method", content = "result")]
pub enum Response<$($generics),*> {
    $(
    $(#[doc = $method_docs])*
    $method_name($transport_return_type),
    )*
}

impl<$($generics),*>  Response<$($generics),*> {
    fn fn_name(&self) -> &'static str {
        match self {
            $(Response::$method_name(..) => stringify!($method_name), )*
        }
    }
}

$(#[doc = $docs])*
///
/// This is the trait which is used by the server side in order to serve the client
pub trait [<$name Server>]<$($generics),*>: MaybeSend + MaybeSync {
    $(
    $(#[doc = $method_docs])*
    fn $method_name(&self, $($arg_name: __add_stream_bounds!(( + MaybeSend), $($arg_user_type)*)),*) -> impl Future<Output=__add_stream_bounds!(( + MaybeSend), $($user_return_type)*)> + MaybeSend;
    )*
}

/// A [Handler](Handler) which handles requests/responses for a given service
#[derive(Debug, Clone)]
pub struct [<$name Handler>]<_Server $(,$generics)*>(_Server, PhantomData<fn() -> ($($generics,)*)>);

impl<_Server: [<$name Server>]<$($generics,)*>, _Format, $($generics,)*> Handler<_Format> for [<$name Handler>]<_Server, $($generics),*>
where
    ():Sized,
    $($(_Format: ReadFormat<$streamed_in>,)*)*
    $($(_Format: WriteFormat<$streamed_out>,)*)*
    $($($streamed_in: MaybeSync,)*)*
    $($($streamed_out: MaybeSync,)*)*
    $($generics: MaybeSend + 'static,)*
    $($where_bounds)*
{
    type Rpc = $name<$($generics),*>;
    type Server = _Server;

    fn server(&self) -> &Self::Server {
        &self.0
    }

    async fn handle<'a>(
        &'a self,
        request: Request<$($generics),*>,
        controller: &impl ConnectionController<'a, _Format>
    ) -> Response<$($generics),*> {
        match request {
            $(
            Request::$method_name { $($arg_name),* } => {
                $(let $arg_name = __streams!(receiving_stream, $arg_name, controller, $($arg_user_type)*);)*
                let value = self.0.$method_name($($arg_name),*).await;
                let value = __streams!(sending_stream, value, controller, $($user_return_type)*);
                Response::$method_name(value)
            }
            )*
        }
    }
}

pub trait [<$name Client>]<$($generics),*>: Send + Sync where ():Sized, $($where_bounds)* {
    type Error: std::fmt::Debug;
    $(
    $(#[doc = $method_docs])*
    fn $method_name(&self, $($arg_name: __add_stream_bounds!(( + MaybeSend + 'static), $($arg_user_type)*)),*) -> impl Future<Output=Result<__add_stream_bounds!(( + MaybeSend), $($user_return_type)*), Self::Error>> + MaybeSend;
    )*
}

pub struct [<$name Client Impl>]<_Conn, _Format, $($generics),*>(Client<_Conn, _Format>, PhantomData<fn() -> ($($generics,)*)>) where ():Sized, $($where_bounds)*;

$(#[doc = $docs])*
///
/// This is the async client for the service, it produces requests from method calls
/// and sends the requests with the given [client](Client) before returning the response
///
///  The return value is always wrapped in a result: `Result<T, ClientError<_Conn::SendError>>` where `T` is the service return value
impl<_Conn, _Format, $($generics),*> [<$name Client>]<$($generics)*> for [<$name Client Impl>]<_Conn, _Format, $($generics),*>
where
    _Conn: Connection + MaybeSend + 'static,
    _Format: Format<Response<$($generics),*>, Request<$($generics),*>>,
    $($(_Format: ReadFormat<$streamed_out>,)*)*
    $($(_Format: WriteFormat<$streamed_in>,)*)*
    $($($streamed_in: MaybeSync,)*)*
    $($($streamed_out: MaybeSync,)*)*
    $($generics: Send + 'static,)*
    $($where_bounds)* {
    type Error = ClientError<_Conn::SendError>;

    $(
    $(#[doc = $method_docs])*
    async fn $method_name(&self, $($arg_name: __add_stream_bounds!(( + MaybeSend + 'static), $($arg_user_type)*)),*) -> Result<$($user_return_type)*, Self::Error> {
        $(let $arg_name = __streams!(sending_stream, $arg_name, self.0, $($arg_user_type)*);)*
        match self.0.send(Request::$method_name { $($arg_name),* }).await? {
            Response::$method_name(value) => {
                let value = __streams!(receiving_stream, value, self.0, $($user_return_type)*);
                Ok(value)
            }
            other => Err(WrongResponseType::new(stringify!($method_name), other.fn_name()).into())
        }
    }
    )*
}

}
}
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __streams {
    // tuple
    ($method:ident, $value:ident, $controller:expr, ($($t:ty),*$(,)?)) => {
        $crate::__streams!(tuple: $method, $value, $controller, ($($t,)*))
    };
    (tuple: $method:ident, $value:ident, $controller:expr, ($($t:ty,)*)) => {
        $crate::__streams!(named_tuple: $method, $value, $controller, a, ($($t,)*))
    };
    (named_tuple: $method:ident, $value:ident, $controller:expr, $i:tt, ($(#$name:tt: $named:ty,)*$t:ty,$($unnamed:ty,)*)) => {
        $crate::__streams!(named_tuple: $method, $value, $controller, [<$i a>], ($(#$name: $named,)* #$i: $t, $($unnamed,)*))
    };
    (named_tuple: $method:ident, $value:ident, $controller:expr, $_:tt, ($(#$name:tt: $t:ty,)*)) => {{
        let ($($name,)*) = $value;
        $(
        let $name = $crate::__streams!($method, $name, $controller, $t);
        )*
        ($($name,)*)
    }};
    ($method:ident, $value:ident, $controller:expr, Result<$ok:ty, $err:ty>) => {
        match $value {
            Ok($value) => {
                Ok($crate::__streams!($method, $value, $controller, $ok))
            },
            Err($value) => {
                Err($crate::__streams!($method, $value, $controller, $err))
            },
        }
    };
    ($method:ident, $value:ident, $controller:expr, Option<$ty:ty>) => {
        match $value {
            Some($value) => {
                Some($crate::__streams!($method, $value, $controller, $ty))
            },
            None => None
        }
    };
    ($method:ident, $value:ident, $controller:expr, impl Stream<Item = $item:ty> $($additional_bounds:tt)*) => {
        $controller.$method::<$item>($value).await
    };
    ($method:ident, $value:ident, $controller:expr, impl futures::stream::Stream<Item = $item:ty> $($additional_bounds:tt)*) => {
        $controller.$method::<$item>($value).await
    };
    ($method:ident, $value:ident, $controller:expr, impl $($unexpected:tt)*) => {
        compile_error!(concat!("unexpected impl argument: impl ", stringify!($($unexpected)*)));
    };
    ($method:ident, $value:ident, $controller:expr, $other:ty) => {
        $value
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
