//! Defines the panic handling strategy, this is how the framework reacts to panics in server
//! functions/streams
//!
//! The available options are:
//! - [`Catch`]: Catch panics with compiler checked unwind safety (default)
//! - [`CatchAssertSafe`]: Catch panics without compiler checked unwind safety
//! - [`Unwind`]: Do not catch panics

use std::future::ready;
use std::panic::{catch_unwind, RefUnwindSafe, AssertUnwindSafe};
use futures::future::BoxFuture;
use crate::connection::message::{HandleError, RequestID};
use crate::connection::{MethodID, ServiceManager, TeardownGuard};
use crate::Handler;
use crate::runtime::AsyncRuntime;

/// Defines a panic strategy
///
/// Allows a handler to be wrapped in a wrapper that handles panics
pub trait PanicStrategy<R: AsyncRuntime, H: Handler<R>> {
    /// The wrapper type
    type Wrapped: Handler<R>;
    /// Returns the wrapped handler
    fn wrap_handler(handler: H) -> Self::Wrapped;
}



/// Catches any panics and sends a [`HandleError::InternalError`] to the client and continue serving the service
///
/// This requires the handler to be bound by [`UnwindSafe`](std::panic::UnwindSafe), meaning that the compiler will ensure
/// there is no value crossing the unwind boundary that could break logical invariants, this is
/// safe, but not always practical, if you want the compiler to ignore unwind safety use
/// [`CatchAssertSafe`]
///
/// see [`panic_handler`](self) for all options
pub struct Catch;

impl<R, H> PanicStrategy<R, H> for Catch
where
    R: AsyncRuntime,
    H: Handler<R> + RefUnwindSafe
{
    type Wrapped = CatchHandler<H>;

    fn wrap_handler(handler: H) -> Self::Wrapped {
        CatchHandler { handler }
    }
}

#[doc(hidden)]
pub struct CatchHandler<H> {
    handler: H,
}

impl<R, H> Handler<R> for CatchHandler<H>
where
    R: AsyncRuntime,
    H: Handler<R> + RefUnwindSafe
{
    fn handle(&self, request_id: RequestID, method_id: MethodID, request: Vec<u8>, stream_handler: ServiceManager<R>, guard: TeardownGuard) -> BoxFuture<'_, Result<Vec<u8>, HandleError>> {
        let result = catch_unwind(move || {
            self.handler.handle(request_id, method_id, request, stream_handler, guard)
        });
        result.unwrap_or_else(|_panic| {
            Box::pin(ready(Err(HandleError::InternalError("Server paniced, see server logs for details".to_string()))))
        })
    }
}

/// Catch any panics and sends a [`HandleError::InternalError`] to the client and continue serving the service
///
/// This uses [`AssertUnwindSafe`] to assert that the handler can be unwound, memory safety is
/// still guaranteed, but the user is responsible to ensure that logical safety is maintained across
/// unwinds, the compiler will not check
///
/// see [`panic_handler`](super) for all options
pub struct CatchAssertSafe;

impl<R, H> PanicStrategy<R, H> for CatchAssertSafe
where
    R: AsyncRuntime,
    H: Handler<R>
{
    type Wrapped = CatchAssertSafeHandler<H>;

    fn wrap_handler(handler: H) -> Self::Wrapped {
        CatchAssertSafeHandler { handler: AssertUnwindSafe(handler) }
    }
}

#[doc(hidden)]
pub struct CatchAssertSafeHandler<H> {
    handler: AssertUnwindSafe<H>,
}

impl<R: AsyncRuntime, H: Handler<R>> crate::Handler<R> for CatchAssertSafeHandler<H> {
    fn handle(&self, request_id: RequestID, method_id: MethodID, request: Vec<u8>, stream_handler: ServiceManager<R>, guard: TeardownGuard) -> BoxFuture<'_, Result<Vec<u8>, HandleError>> {
        let result = catch_unwind(move || {
            self.handler.handle(request_id, method_id, request, stream_handler, guard)
        });
        result.unwrap_or_else(|_panic| {
            Box::pin(ready(Err(HandleError::InternalError("Server paniced, see server logs for details".to_string()))))
        })
    }
}

/// Panics will function as in normal rust, they will unwind the stack and the entire server will
/// be terminated
///
/// see [`panic_handler`](super) for all options
pub struct Unwind;

impl<R, H> PanicStrategy<R, H> for Unwind
where
    R: AsyncRuntime,
    H: Handler<R>
{
    type Wrapped = H;

    fn wrap_handler(handler: H) -> Self::Wrapped {
        handler
    }
}