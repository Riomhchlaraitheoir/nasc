//! Defines the [`Client`] type which is used to send requests and receive responses

use crate::__macros::imports::StreamID;
use crate::connection::TeardownGuard;
use crate::connection::message::{HandleError, Message, RequestID, ServiceID, StreamMessage};
use crate::connection::{ConnectionManager, MethodID, ServiceManager};
use async_channel::{Receiver, SendError};
use futures::channel::oneshot;
use futures::future::{BoxFuture, Either, FusedFuture, Map, Select, select};
use futures::{FutureExt, Stream};
use pin_project::{pin_project, pinned_drop};
use std::error::Error;
use std::marker::PhantomData;
use std::mem;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use futures_timer::Delay;
use thiserror::Error;
use tracing::error;
use crate::{assert_future, Service};

/// A client which sends requests on the connection and awaits responses
///
/// Generic `S` indicates the service this client is for, purely for type safety
pub struct Client<S: Service> {
    pub(super) manager: Arc<ConnectionManager>,
    pub(super) service_id: ServiceID,
    #[doc(hidden)]
    pub guard: TeardownGuard,
    pub(super) default_timeout: Option<Duration>,
    pub(crate) _service: PhantomData<S>,
}

impl<S: Service> Client<S> {
    /// Send a request and wait for the response
    ///
    /// # Errors
    /// returns an error if the send fails or if the server encounters an issue
    pub fn send<T>(
        &self,
        payload: impl Future<Output = Result<Vec<u8>, ClientError>> + Send + 'static,
        method_id: MethodID,
        response: impl AsyncFnOnce(Vec<u8>) -> Result<T, ClientError>
    ) -> ClientResponseFuture<impl Future<Output = Result<T, ClientError>>>
    {
        let request_id = self.manager.request_ids.next();
        let future = CancellableFuture {
            future: self.send_internal(request_id, method_id, payload, response).fuse(),
            manager: self.manager.clone(),
            request_id,
            _guard: self.guard.clone(),
        };
        assert_future::<Result<T, ClientError>>(&future);
        let future = ClientResponseFuture {
            future,
            timeout: self.default_timeout,
        };
        assert_future::<Result<T, ClientError>>(&future);
        future
    }

    async fn send_internal<T>(
        &self,
        request_id: RequestID,
        method_id: MethodID,
        payload: impl Future<Output = Result<Vec<u8>, ClientError>> + Send + 'static,
        response: impl AsyncFnOnce(Vec<u8>) -> Result<T, ClientError>
    ) -> Result<T, ClientError> {
        let (sender, receiver) = oneshot::channel();
        self.manager
            .response_waiters
            .insert(request_id, sender)
            .await;
        let Ok(()) = self
            .manager
            .message_sender
            .send(Message::Request {
                service_id: self.service_id,
                request_id,
                method_id,
                payload: payload.await?,
            })
            .await
        else {
            return Err(ClientError::ConnectionClosed);
        };
        let response_bytes = receiver
            .await
            .map_err(|_| ClientError::ConnectionClosed)??;
        response(response_bytes).await
    }

    #[must_use]
    /// Returns a [`ServiceManager`]
    pub fn service_manager(&self) -> ServiceManager {
        ServiceManager(self.manager.clone())
    }
}

impl<S: Service> Drop for Client<S> {
    fn drop(&mut self) {
        self.manager.spawn_send(Message::ServiceDropped {
            service_id: self.service_id,
        });
    }
}

/// This is an error that the client may return after a request
#[derive(Debug, Error)]
pub enum ClientError {
    /// The transport layer returned an error
    #[error("Connection Closed")]
    ConnectionClosed,
    /// The request timed out
    #[error("Request timed out")]
    Timeout,
    /// The transport layer returned an error
    #[error("Unexpected response: {0}")]
    Response(#[from] HandleError),
    /// Failed to serialise the request
    #[error("Failed to serialise the request: {0}")]
    Serialise(Box<dyn Error + Send>),
    /// Failed to deserialise the response
    #[error("Failed to deserialise the response: {0}")]
    Deserialise(Box<dyn Error + Send>),
}

#[pin_project(PinnedDrop)]
pub(super) struct ClientStream {
    #[pin]
    pub(super) receiver: Receiver<Vec<u8>>,
    pub(super) guard: TeardownGuard,
    pub(super) manager: Arc<ConnectionManager>,
    pub(super) stream_id: StreamID,
    pub(super) state: ClientStreamState,
}

pub(super) enum ClientStreamState {
    Ready,
    Polling(BoxFuture<'static, Result<(), SendError<Message>>>),
    Waiting,
    Done,
}

impl Stream for ClientStream {
    type Item = Vec<u8>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let stream = self.project();
        let state = mem::replace(stream.state, ClientStreamState::Waiting);
        let mut send = match state {
            ClientStreamState::Ready => {
                let sender = stream.manager.message_sender.clone();
                let stream_id = *stream.stream_id;
                Box::pin(async move {
                    sender
                        .send(Message::Stream {
                            stream_id,
                            message: StreamMessage::Poll,
                        })
                        .await
                })
            }
            ClientStreamState::Polling(send) => send,
            ClientStreamState::Waiting => {
                return match stream.receiver.poll_next(cx) {
                    Poll::Ready(Some(item)) => {
                        *stream.state = ClientStreamState::Ready;
                        Poll::Ready(Some(item))
                    }
                    Poll::Ready(None) => {
                        *stream.state = ClientStreamState::Done;
                        Poll::Ready(None)
                    }
                    Poll::Pending => Poll::Pending,
                };
            }
            ClientStreamState::Done => return Poll::Ready(None),
        };
        match send.poll_unpin(cx) {
            Poll::Ready(Ok(())) => match stream.receiver.poll_next(cx) {
                Poll::Ready(Some(item)) => {
                    *stream.state = ClientStreamState::Ready;
                    Poll::Ready(Some(item))
                }
                Poll::Ready(None) => {
                    *stream.state = ClientStreamState::Done;
                    Poll::Ready(None)
                }
                Poll::Pending => Poll::Pending,
            },
            Poll::Ready(Err(error)) => {
                error!(
                    "Failed to send poll message, ending stream early: {}",
                    error
                );
                Poll::Ready(None)
            }
            Poll::Pending => {
                *stream.state = ClientStreamState::Polling(send);
                Poll::Pending
            }
        }
    }
}

#[pinned_drop]
impl PinnedDrop for ClientStream {
    fn drop(self: Pin<&mut Self>) {
        self.manager.spawn_send(Message::Stream {
            stream_id: self.stream_id,
            message: StreamMessage::Dropped,
        });
    }
}

/// Future returned from [`Client::send`]
///
/// Will automatically cancel server handling if the future is dropped before resolving
#[pin_project(PinnedDrop)]
struct CancellableFuture<F: FusedFuture> {
    #[pin]
    future: F,
    manager: Arc<ConnectionManager>,
    request_id: RequestID,
    _guard: TeardownGuard,
}

impl<F: FusedFuture> Future for CancellableFuture<F> {
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().future.poll(cx)
    }
}

#[pinned_drop]
impl<F: FusedFuture> PinnedDrop for CancellableFuture<F> {
    fn drop(self: Pin<&mut Self>) {
        if self.future.is_terminated() {
            return;
        }
        self.manager.spawn_send(Message::CancelRequest {
            request_id: self.request_id,
        });
    }
}

/// A future which fetches a response for a client
///
/// Timeout can be modified using [`timeout`](Self::timeout) or [`no_timeout`](Self::no_timeout)
pub struct ClientResponseFuture<F> {
    future: F,
    timeout: Option<Duration>,
}

impl<F> ClientResponseFuture<F> {
    /// Set the timeout for making this request call
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Removes the timeout from this request
    #[must_use]
    pub fn no_timeout(mut self) -> Self {
        self.timeout = None;
        self
    }
}

impl<T, F: Future<Output = Result<T, ClientError>>> IntoFuture
    for ClientResponseFuture<F>
{
    type Output = Result<T, ClientError>;
    type IntoFuture = Either<
        F,
        Map<
            Select<Pin<Box<F>>, Delay>,
            fn(<Select<Pin<Box<F>>, Delay> as Future>::Output) -> Result<T, ClientError>,
        >,
    >;

    fn into_future(self) -> Self::IntoFuture {
        if let Some(timeout) = self.timeout {
            Either::Right(
                select(Box::pin(self.future), Delay::new(timeout))
                    .map(inline_timeout::<T, Delay, F>),
            )
        } else {
            Either::Left(self.future)
        }
    }
}

#[allow(clippy::type_complexity, reason="only used here, not part of api")]
fn inline_timeout<T, S, F>(
    either: Either<(Result<T, ClientError>, S), ((), Pin<Box<F>>)>,
) -> Result<T, ClientError> {
    match either {
        Either::Left((result, _)) => result,
        Either::Right(_) => Err(ClientError::Timeout),
    }
}
