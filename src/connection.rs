//! Defines types for managing an active connection with any amount of clients/handlers using the connection

pub mod channel;
pub mod client;
pub mod message;
pub mod sink_stream;

use crate::Handler;
use crate::__macros::imports::Stream;
use crate::connection::client::{Client, ClientStream, ClientStreamState};
use crate::connection::message::{
    ConnectionError, HandleError, Message, RequestID, ServiceID, StreamID, StreamMessage,
};
use crate::spawner::Spawner;
use async_channel::{Receiver, Sender, bounded, unbounded};
use async_lock::Mutex;
use derive_more::Display;
use futures::channel::oneshot;
use futures::future::{AbortHandle, Abortable, Either, FusedFuture};
use futures::stream::BoxStream;
use futures::{FutureExt, StreamExt};
use log::warn;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::pin::{Pin, pin};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::task::{Context, Poll};
use tracing::debug;
use whirlwind::ShardMap;

/// Identifies a certain request
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, Display, Ord, PartialOrd)]
#[cfg_attr(
    all(test, not(target_arch = "wasm32")),
    derive(proptest_derive::Arbitrary)
)]
pub struct MethodID(u32);

impl MethodID {
    #[must_use]
    #[doc(hidden)]
    pub const fn _new(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    #[doc(hidden)]
    pub const fn value(&self) -> u32 {
        self.0
    }
}

#[derive(Debug)]
struct Series<T>(AtomicU32, PhantomData<T>);

impl<T> Series<T> {
    const fn new(is_server: bool) -> Self {
        let start = if is_server { 2 } else { 1 };
        Self(AtomicU32::new(start), PhantomData)
    }
}

impl<T: From<u32>> Series<T> {
    fn next(&self) -> T {
        self.0.fetch_add(2, Ordering::Relaxed).into()
    }
}

/// Represents the underlying connection
pub trait Connection {
    /// Run the connection, handling incoming and outgoing messages
    fn run(
        self,
        driver: ConnectionDriver,
        spawner: impl Spawner + 'static,
    ) -> impl Future<Output = Result<(), impl Display>> + 'static;
}

/// A signal that the connection is closed and the manager should drop
pub struct ConnectionClosed;

struct ConnectionManager {
    response_waiters: ShardMap<RequestID, ResponseWaiter>,
    incoming_streams: ShardMap<StreamID, Sender<Vec<u8>>>,
    service_ids: Series<ServiceID>,
    stream_ids: Series<StreamID>,
    request_ids: Series<RequestID>,
    handlers: ShardMap<ServiceID, (Arc<dyn Handler + Send + Sync>, TeardownGuard)>,
    streams: ShardMap<StreamID, ServerStream>,
    message_sender: Sender<Message>,
    priority_message_sender: Sender<Message>,
    pending_streams: ShardMap<StreamID, AbortHandle>,
    pending_handles: ShardMap<RequestID, AbortHandle>,
    is_server: bool,
}

type ServerStream = (Arc<Mutex<BoxStream<'static, Vec<u8>>>>, TeardownGuard);

type ResponseWaiter = oneshot::Sender<Result<Vec<u8>, HandleError>>;

/// Creates a new [`Client`] for the RPC connection,
/// also returns a [`ConnectionDriver`] to run the event loop
#[allow(
    clippy::unused_async,
    reason = "Allow for future async and maintain symmetry with new_server_conn"
)]
pub async fn new_client_conn() -> (Client, ConnectionDriver) {
    let (driver, teardown_guard) = new_driver(true);
    let client = Client {
        manager: driver.manager.clone(),
        service_id: 0.into(),
        guard: teardown_guard,
    };
    (client, driver)
}

/// Creates a new Server for the RPC connection from the given handler,
/// also returns a [`ConnectionDriver`] to run the event loop
pub async fn new_server_conn(handler: impl Handler + 'static) -> ConnectionDriver {
    let (driver, teardown_guard) = new_driver(true);
    driver
        .manager
        .handlers
        .insert(0.into(), (Arc::new(handler), teardown_guard))
        .await;
    driver
}

fn new_driver(is_server: bool) -> (ConnectionDriver, TeardownGuard) {
    let (message_sender, message_receiver) = bounded(32);
    let (priority_message_sender, priority_message_receiver) = unbounded();
    let (teardown_guard, teardown_signal) = new_teardown();
    let manager = Arc::new(ConnectionManager {
        response_waiters: ShardMap::new(),
        incoming_streams: ShardMap::new(),
        service_ids: Series::new(is_server),
        stream_ids: Series::new(is_server),
        request_ids: Series::new(is_server),
        handlers: ShardMap::new(),
        streams: ShardMap::new(),
        message_sender,
        priority_message_sender,
        pending_streams: ShardMap::new(),
        pending_handles: ShardMap::new(),
        is_server,
    });
    let driver = ConnectionDriver {
        manager: Arc::clone(&manager),
        message_receiver,
        priority_message_receiver,
        teardown_signal: Mutex::new(teardown_signal),
    };
    (driver, teardown_guard)
}

impl ConnectionManager {
    async fn send_message(&self, message: Message) {
        if self.message_sender.send(message).await.is_err() {
            warn!("Connection closed");
        }
    }

    async fn handle_message(
        self: &Arc<Self>,
        message: Message,
        executor: &impl Spawner,
    ) -> Result<(), ConnectionError> {
        debug!("Received message: {:?}", message);
        match message {
            Message::Request {
                service_id,
                request_id,
                method_id,
                payload,
            } => {
                if let Some((handler, guard)) =
                    self.handlers.get(&service_id).await.map(|h| h.clone())
                {
                    let message_sender = self.message_sender.clone();
                    let manager = Arc::clone(self);
                    let pending_handles = self.pending_handles.clone();
                    let task = async move {
                        let result = handler
                            .handle(
                                request_id,
                                method_id,
                                payload,
                                ServiceManager(manager),
                                guard,
                            )
                            .await;
                        if let Err(error) = message_sender
                            .send(Message::Response { request_id, result })
                            .await
                        {
                            warn!("Connection closed: {error}");
                        }
                        pending_handles.remove(&request_id).await;
                    };
                    let (abort_handle, abort_registration) = AbortHandle::new_pair();
                    self.pending_handles.insert(request_id, abort_handle).await;
                    executor.spawn(Abortable::new(task, abort_registration));
                } else {
                    warn!("Service {service_id} not found");
                    self.send_message(Message::Response {
                        request_id,
                        result: Err(HandleError::ServiceNotFound),
                    })
                    .await;
                }
            }
            Message::CancelRequest { request_id } => {
                if let Some(handle) = self.pending_handles.remove(&request_id).await {
                    handle.abort();
                }
            }
            Message::Response { request_id, result } => {
                if let Some(sender) = self.response_waiters.remove(&request_id).await {
                    // error indicates client was dropped, ignore
                    let _ = sender.send(result);
                }
            }
            Message::ServiceDropped { service_id } => {
                if self.is_server == service_id.is_server_side() {
                    // if service is served by the local peer
                    if self.handlers.remove(&service_id).await.is_none() {
                        warn!("Service {service_id} not found");
                    }
                } else {
                    todo!("Close client somehow")
                }
            }
            Message::ConnectionError(error) => return Err(error),
            Message::Stream { stream_id, message } => {
                self.handle_stream_message(stream_id, message, executor)
                    .await;
            }
        }
        Ok(())
    }

    async fn handle_stream_message(
        self: &Arc<Self>,
        stream_id: StreamID,
        message: StreamMessage,
        executor: &impl Spawner,
    ) {
        match message {
            StreamMessage::Poll => {
                if let Some((stream, guard)) = self.streams.get(&stream_id).await.map(|s| s.clone())
                {
                    let message_sender = self.message_sender.clone();
                    let pending_streams = self.pending_streams.clone();
                    let task = async move {
                        let _guard = guard; // move guard into future to prevent connection dropping
                        let item = stream.lock().await.next().await;
                        if let Err(error) = message_sender
                            .send(Message::Stream {
                                stream_id,
                                message: StreamMessage::Item(item),
                            })
                            .await
                        {
                            warn!("Connection closed: {error}");
                        }
                        pending_streams.remove(&stream_id).await;
                    };
                    let (abort_handle, abort_registration) = AbortHandle::new_pair();
                    self.pending_streams.insert(stream_id, abort_handle).await;
                    executor.spawn(Abortable::new(task, abort_registration));
                } else {
                    warn!("Stream {stream_id} not found, sending Done");
                    if let Err(error) = self
                        .message_sender
                        .send(Message::Stream {
                            stream_id,
                            message: StreamMessage::Item(None),
                        })
                        .await
                    {
                        warn!("Connection closed: {error}");
                    }
                }
            }
            StreamMessage::Item(item) => {
                if let Some(item) = item {
                    if let Some(sender) = self
                        .incoming_streams
                        .get(&stream_id)
                        .await
                        .map(|lock_guard| lock_guard.clone())
                    {
                        // Err occurs when client stream is dropped, ignore
                        let _ = sender.send(item).await;
                    } else {
                        warn!("Stream {stream_id} not found");
                    }
                } else {
                    if let Some(sender) = self.incoming_streams.remove(&stream_id).await {
                        sender.close();
                    } else {
                        warn!("Stream {stream_id} not found");
                    }
                }
            }
            StreamMessage::Dropped => {
                self.streams.remove(&stream_id).await;
                if let Some(handle) = self.pending_streams.remove(&stream_id).await {
                    handle.abort();
                }
            }
        }
    }
}

/// Allows creation of new services/streams
pub struct ServiceManager(Arc<ConnectionManager>);

impl ServiceManager {
    /// Create a new outgoing service
    pub async fn outgoing_service(
        &self,
        handler: impl Handler + 'static,
        guard: TeardownGuard,
    ) -> ServiceID {
        let service_id = self.0.service_ids.next();
        debug!("Outgoing service {service_id}");
        self.0
            .handlers
            .insert(service_id, (Arc::new(handler), guard))
            .await;
        service_id
    }

    /// Create a new incoming service
    #[allow(
        clippy::unused_async,
        reason = "Keep this async for forward compatability and consistency with other similar methods"
    )]
    pub async fn incoming_service(&self, service_id: ServiceID, guard: TeardownGuard) -> Client {
        debug!("Received incoming service {service_id}");
        Client {
            manager: Arc::clone(&self.0),
            service_id,
            guard,
        }
    }

    /// Create a new outgoing stream
    pub async fn outgoing_stream(
        &self,
        stream: impl Stream<Item = Vec<u8>> + Send + 'static,
        guard: TeardownGuard,
    ) -> StreamID {
        let stream_id = self.0.stream_ids.next();
        debug!("Outgoing stream {stream_id}");
        self.0
            .streams
            .insert(stream_id, (Arc::new(Mutex::new(Box::pin(stream))), guard))
            .await;
        stream_id
    }

    /// Create a new incoming stream
    pub async fn incoming_stream(
        &self,
        stream_id: StreamID,
        guard: TeardownGuard,
    ) -> impl Stream<Item = Vec<u8>> + Send + 'static + use<> {
        debug!("Received incoming stream {stream_id}");
        let (sender, receiver) = bounded(10);
        self.0.incoming_streams.insert(stream_id, sender).await;
        ClientStream {
            receiver,
            guard,
            manager: Arc::clone(&self.0),
            stream_id,
            state: ClientStreamState::Ready,
        }
    }
}

/// Drives the event loop for the connection
///
/// Manages both waiting for requests/responses and sending requests/responses
///
/// Automatically terminates when the last client/handler is dropped
pub struct ConnectionDriver {
    // TODO: make fields pub and allow Connection impl to drive loop
    manager: Arc<ConnectionManager>,
    message_receiver: Receiver<Message>,
    priority_message_receiver: Receiver<Message>,
    teardown_signal: Mutex<TeardownSignal>,
}

impl ConnectionDriver {
    /// Consume the incoming message and handle it appropriately, this function will spawn a new
    /// task if it does anything that might take some time (calling handler methods, polling streams)
    pub async fn consume_incoming_message(
        &self,
        message: Message,
        executor: &impl Spawner,
    ) -> Result<(), ConnectionError> {
        self.manager.handle_message(message, executor).await
    }

    /// Get all remaining message that are waiting to be sent, should be used after teardown signal
    /// to ensure remote peer receives all outstanding notices of dropped streams/services to ensure
    /// it can close safely too
    pub fn flush_outgoing_messages(&self) -> impl Iterator<Item = Message> {
        std::iter::from_fn(|| self.priority_message_receiver.try_recv().ok())
            .chain(std::iter::from_fn(|| self.message_receiver.try_recv().ok()))
    }

    /// Returns the next outgoing message, may take some time to actually return if no message is
    /// currently queued
    ///
    /// Returns None if there are no more messages to send, this is not expected to happen prior
    /// to the teardown signal
    pub async fn next_outgoing_message(&self) -> Option<Message> {
        let next_priority = pin!(self.priority_message_receiver.recv());
        let next = pin!(self.message_receiver.recv());
        let (Either::Left((result, _)) | Either::Right((result, _))) =
            futures::future::select(next_priority, next).await;
        result.ok()
    }

    /// This future returns when all clients/handlers have been dropped, this signals that the
    /// connection is no longer needed and can be closed
    pub async fn teardown(&self) {
        (&mut *self.teardown_signal.lock().await).await;
    }
}

fn new_teardown() -> (TeardownGuard, TeardownSignal) {
    let (teardown_guard, teardown_signal) = oneshot::channel::<()>();
    (
        TeardownGuard(Arc::new(teardown_guard)),
        TeardownSignal(teardown_signal),
    )
}

/// A guard that ensures the [`ConnectionDriver`] stays running,
///
/// When this guard and all of its clones are dropped, the `ConnectionDriver` will stop running and
/// the underlying `ConnectionManager` will be dropped
#[clippy::has_significant_drop]
#[derive(Clone)]
pub struct TeardownGuard(
    #[allow(
        unused,
        reason = "This is never used directly, instead it closes the channel when all `Arc`s have been dropped, signalling teardown"
    )]
    Arc<oneshot::Sender<()>>,
);

struct TeardownSignal(oneshot::Receiver<()>);

impl Future for TeardownSignal {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // expect the channel to simply be closed, so ignoring result
        self.0.poll_unpin(cx).map(|_| ())
    }
}

impl FusedFuture for TeardownSignal {
    fn is_terminated(&self) -> bool {
        self.0.is_terminated()
    }
}
