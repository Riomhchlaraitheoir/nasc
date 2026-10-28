use std::panic::AssertUnwindSafe;
use crate::Handler;
use crate::connection::message::{HandleError, RequestID};
use crate::connection::{MethodID, ServiceManager, TeardownGuard};
use futures::future::BoxFuture;
use futures::FutureExt;

#[derive(Clone)]
pub struct RegisteredHandler<H> {
    handler: H,
    guard: TeardownGuard,
}

impl<H> RegisteredHandler<H> {
    pub fn new(handler: H, guard: TeardownGuard) -> Self {
        Self {
            handler,
            guard,
        }
    }
}

impl<H: Handler> RawHandler for RegisteredHandler<H> {
    fn handle(
        &self,
        request_id: RequestID,
        method_id: MethodID,
        request: Vec<u8>,
        stream_handler: ServiceManager,
    ) -> BoxFuture<'_, Result<Vec<u8>, HandleError>>
    {
        Box::pin(async move {
            AssertUnwindSafe(self.handler
                .handle(request_id, method_id, request, stream_handler, self.guard.clone()))
                .catch_unwind()
                .await
                .unwrap_or_else(|_| {
                Err(HandleError::InternalError("handler panicked".to_string()))
            })
        })
    }
}

pub trait RawHandler: Send + Sync + 'static {
    /// takes the request and returns a response, see [trait documentation](Self) for details
    fn handle(
        &self,
        request_id: RequestID,
        method_id: MethodID,
        request: Vec<u8>,
        stream_handler: ServiceManager,
    ) -> BoxFuture<'_, Result<Vec<u8>, HandleError>>;
}