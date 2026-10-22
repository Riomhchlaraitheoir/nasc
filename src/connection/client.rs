//! Defines the [`Client`] type which is used to send requests and receive responses

use crate::__macros::imports::StreamID;
use crate::connection::TeardownGuard;
use crate::connection::message::{HandleError, Message, RequestID, ServiceID, StreamMessage};
use crate::connection::{ConnectionManager, MethodID, ServiceManager};
use async_channel::{Receiver, SendError};
use futures::channel::oneshot;
use futures::future::{BoxFuture, FusedFuture};
use futures::{FutureExt, Stream};
use pin_project::{pin_project, pinned_drop};
use std::error::Error;
use std::mem;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use thiserror::Error;
use tracing::error;

/// A client which sends requests on the connection and awaits responses
#[clippy::has_significant_drop]
pub struct Client {
    pub(super) manager: Arc<ConnectionManager>,
    pub(super) service_id: ServiceID,
    #[doc(hidden)]
    pub guard: TeardownGuard,
}

impl Client {
    /// Send a request and wait for the response
    ///
    /// # Errors
    /// returns an error if the send fails or if the server encounters an issue
    pub fn send(
        &self,
        method_id: MethodID,
        payload: Vec<u8>,
    ) -> impl Future<Output = Result<Vec<u8>, ClientError>> {
        let request_id = self.manager.request_ids.next();
        CancellableFuture {
            future: self.send_internal(request_id, method_id, payload).fuse(),
            manager: self.manager.clone(),
            request_id,
            _guard: self.guard.clone(),
        }
    }

    async fn send_internal(
        &self,
        request_id: RequestID,
        method_id: MethodID,
        payload: Vec<u8>,
    ) -> Result<Vec<u8>, ClientError> {
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
                payload,
            })
            .await
        else {
            return Err(ClientError::ConnectionClosed);
        };
        Ok(receiver
            .await
            .map_err(|_| ClientError::ConnectionClosed)??)
    }

    #[must_use]
    /// Returns a [`ServiceManager`]
    pub fn service_manager(&self) -> ServiceManager {
        ServiceManager(self.manager.clone())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        if let Err(error) = self
            .manager
            .priority_message_sender
            .try_send(Message::ServiceDropped {
                service_id: self.service_id,
            })
        {
            error!("Failed to send ServiceDropped message: {}", error);
        }
    }
}

/// This is an error that the client may return after a request
#[derive(Debug, Error)]
pub enum ClientError {
    /// The transport layer returned an error
    #[error("Connection Closed")]
    ConnectionClosed,
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

#[clippy::has_significant_drop]
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
        if let Err(error) = self
            .manager
            .priority_message_sender
            .try_send(Message::Stream {
                stream_id: self.stream_id,
                message: StreamMessage::Dropped,
            })
        {
            error!("Failed to send ServiceDropped message: {}", error);
        }
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
        if let Err(error) = self
            .manager
            .priority_message_sender
            .try_send(Message::CancelRequest {
                request_id: self.request_id,
            })
        {
            error!("Failed to send ServiceDropped message: {}", error);
        }
    }
}
