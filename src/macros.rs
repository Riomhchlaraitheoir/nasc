
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
    };
    pub use ::{
        futures::{
            sink::{Sink, SinkExt},
            stream::{Stream, StreamExt},
            future::BoxFuture,
            Future
        },
        std::marker::PhantomData,
        tracing,
        pastey::paste
    };
}
#[doc(hidden)]
pub mod traits {
    pub use crate::{
        IntoHandler as _,
    };
}
