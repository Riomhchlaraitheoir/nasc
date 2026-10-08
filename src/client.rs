//! Contains modules for individual client implementations
#![allow(
    clippy::future_not_send,
    reason = "Cannot explicitly make futures `Send` while supporting WASM"
)]

use crate::format::{Format, ReadFormat, WriteFormat};
use futures::{Stream, StreamExt};
use std::error::Error;
use std::fmt::{Debug, Display};
use std::future::ready;
use async_channel::{Receiver, Sender};
use futures::channel::oneshot;
use futures::channel::oneshot::Canceled;
use futures::stream::unfold;
use thiserror::Error;
use tracing::{error, warn};
use whirlwind::ShardMap;

#[cfg(all(feature = "browser", not(target_arch = "wasm32")))]
compile_error!("browser feature is only available for wasm32 target arch");

#[cfg(tokio_websocket)]
pub mod tokio_websocket;
#[cfg(tokio_websocket)]
pub use tokio_websocket as websocket;

#[cfg(wasm_websocket)]
pub mod wasm_websocket;
#[cfg(wasm_websocket)]
pub use wasm_websocket as websocket;

use crate::{BoxStream, Executor, MaybeSend, MaybeSync, Rc, Series};
use crate::message::{ConnectionError, ConnectionMessage, RequestID, StreamID};

/// This is a error that the client may return after a request
#[derive(Debug, Error)]
pub enum RpcError<T> {
    /// The transport layer returned an error
    #[error("Failed to send request: {0}")]
    Transport(#[source] T),
    /// The transport layer returned an error
    #[error("Unexpected response: {0}")]
    Response(#[from] HandleError),
    /// Failed to serialize the request
    #[error("Failed to serialize the request: {0}")]
    Serialize(Box<dyn Error + Send>),
    /// Failed to deserialize the response
    #[error("Failed to deserialize the response: {0}")]
    Deserialize(Box<dyn Error + Send>),
    /// Response was the wrong type, sent a request for one function, but received the response of a different one
    ///
    /// This is not an expected case and is simply included as an alternative to panicking in this case
    /// This error either means the server side is misbehaving quite badly, or the transport is not configured to the correct endpoint
    #[error(transparent)]
    WrongResponseType(#[from] WrongResponseType),
}

/// Indicates that the transport was successful, but the response indicated some problem
#[derive(Debug, Error, Clone)]
pub enum HandleError {
    /// Request was improperly formatted
    #[error("Request was rejected: {0}")]
    BadRequest(String),
    /// Internal Server Error
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
    /// Unexpected response
    #[error("Unexpected response")]
    Unexpected,
}

/// Response was the wrong type: sent a request for one function, but received the response of a different one
///
/// This is not an expected case and is simply included to avoid panicking in this case
/// This error either means the server side is misbehaving quite badly, or the transport is not configured to the correct endpoint
#[derive(Debug, Error, Clone)]
#[error(
    "Response was for the wrong method, sent a request for {expected}, but received the response for {actual}"
)]
pub struct WrongResponseType {
    /// The expected method
    pub expected: String,
    /// The actual method
    pub actual: String,
}

impl WrongResponseType {
    #[doc(hidden)]
    #[must_use]
    pub fn new(expected: &str, actual: &str) -> Self {
        Self {
            expected: format!("{expected}()"),
            actual: format!("{actual}()"),
        }
    }
    #[doc(hidden)]
    #[must_use]
    pub fn in_subservice(self, name: &str) -> Self {
        Self {
            expected: format!("{name}().{}", self.expected),
            actual: format!("{name}().{}", self.actual),
        }
    }
}

/// An error which might be a [`WrongResponseType`] error
pub trait MaybeWrongResponse: Sized {
    /// Try to cast to a [`WrongResponseType`]
    ///
    /// # Errors
    ///
    /// Returns an Err variant with self if this value is not a [`WrongResponseType`]
    fn into_wrong_response(self) -> Result<WrongResponseType, Self>;
}

impl<T: Error> MaybeWrongResponse for RpcError<T> {
    fn into_wrong_response(self) -> Result<WrongResponseType, Self> {
        if let Self::WrongResponseType(err) = self {
            Ok(err)
        } else {
            Err(self)
        }
    }
}

impl MaybeWrongResponse for WrongResponseType {
    fn into_wrong_response(self) -> Result<WrongResponseType, Self> {
        Ok(self)
    }
}


/// Represents the underlying connection
pub trait Connection: MaybeSync {
    /// An error that can occur when sending on this connection
    type SendError: Debug + Display + MaybeSend + 'static;
    /// An error that can occur when reading from this connections
    type RecvError: Debug + Display + MaybeSend + 'static;

    /// Send the given raw bytes to the connection
    fn send(&self, value: Vec<u8>) -> impl Future<Output = Result<(), Self::SendError>> + MaybeSend;
    /// Read raw bytes from the connection
    fn recv(&self) -> impl Future<Output = Result<Vec<u8>, Self::RecvError>> + MaybeSend;
}

/// This type manages a connection (either client-side or server-side), manages all incoming and
/// outgoing streams and allows higher layers to focus on request/response without dealing without
/// the streams
///
/// NOTE: This will only work as long as [receive](Self::receive) is regularly polled
struct ConnectionManager<Conn> {
    conn: Rc<Conn>,
    senders: ShardMap<RequestID, oneshot::Sender<Result<Vec<u8>, HandleError>>>,
    outgoing_streams:
        ShardMap<StreamID, BoxStream<'static, Vec<u8>>>,
    polled_incoming_streams: ShardMap<StreamID, oneshot::Sender<Option<Vec<u8>>>>,
    request_ids: Rc<Series<RequestID>>,
    stream_ids: Rc<Series<StreamID>>,
}

impl<Conn> Clone for ConnectionManager<Conn> {
    fn clone(&self) -> Self {
        Self {
            conn: self.conn.clone(),
            senders: self.senders.clone(),
            outgoing_streams: self.outgoing_streams.clone(),
            polled_incoming_streams: self.polled_incoming_streams.clone(),
            request_ids: self.request_ids.clone(),
            stream_ids: self.stream_ids.clone(),
        }
    }
}

impl<Conn> ConnectionManager<Conn>
where
    Conn: Connection + MaybeSync + MaybeSend + 'static,
{
    pub fn new(conn: Conn) -> Self {
        Self {
            conn: Rc::new(conn),
            senders: ShardMap::new(),
            outgoing_streams: ShardMap::new(),
            polled_incoming_streams: ShardMap::new(),
            request_ids: Rc::default(),
            stream_ids: Rc::default(),
        }
    }

    pub(crate) async fn start(self: Rc<Self>) -> Result<(), ConnectionError> {
        let executor = Executor::new();

        loop {
            let Ok(next) = self.conn.recv().await else {
                break Ok(())
            };
            let next = ConnectionMessage::parse(&next);
            let (request_id, result) = match next {
                ConnectionMessage::ConnectionError(error) => {
                    break Err(error);
                }
                ConnectionMessage::Payload { request_id, payload } => {
                    (request_id, Ok(payload))
                }
                ConnectionMessage::HandleError { request_id, error } => {
                    (request_id, Err(error))
                }
                ConnectionMessage::StreamItem { stream_id, item } => {
                    if let Some(sender) = self.polled_incoming_streams.remove(&stream_id).await {
                        // error indicates handling function dropped stream, no longer interested
                        let _: Result<_, Option<Vec<u8>>> = sender.send(item);
                    } else {
                        warn!("stream {stream_id} was not polled");
                    }
                    continue
                }
                ConnectionMessage::StreamPoll { stream_id } => {
                    let outgoing_streams = &self.outgoing_streams;
                    let conn = &self.conn;
                    executor
                        .spawn(async move {
                            let item = if let Some(mut stream) =
                                outgoing_streams.remove(&stream_id).await
                            {
                                let item = stream.next().await;
                                outgoing_streams.insert(stream_id, stream).await;
                                item
                            } else {
                                warn!("Stream {stream_id} not found");
                                None
                            };
                            let result = conn
                                .send(ConnectionMessage::StreamItem { stream_id, item }.format())
                                .await;
                            if let Err(error) = result {
                                error!("Error while sending message: {error:?}");
                            }
                        })
                        .detach();
                    continue
                }
            };
            let Some(sender) = self.senders.remove(&request_id).await else {
                warn!("Received payload for unexpected request: {request_id}");
                continue;
            };
            // error occurs only if the future awaiting the response was dropped, ignore
            // this as expected
            let _: Result<(), Result<Vec<u8>, HandleError>> = sender.send(result);
        }
    }

    /// Send a message on the connection
    async fn send(&self, message: Vec<u8>) -> Result<Result<Vec<u8>, HandleError>, SendError<Conn::SendError>> {
        let request_id = self.request_ids.next();
        let message = ConnectionMessage::Payload {
            request_id,
            payload: message,
        };
        let (sender, receiver) = oneshot::channel();
        self.senders.insert(request_id, sender).await;
        self.conn.send(message.format()).await.map_err(SendError::Conn)?;
        Ok(receiver.await?)
    }

    fn receiving_stream(&self, id: StreamID) -> impl Stream<Item=Vec<u8>> + MaybeSend {
        unfold(id, async |id| {
            let (sender, receiver) = oneshot::channel();
            self.polled_incoming_streams.insert(id, sender).await;

            let result: Result<(), _> = self.conn
                .send(ConnectionMessage::StreamPoll { stream_id: id }.format())
                .await;
            if let Err(error) = result {
                error!("Error while sending message: {error:?}");
            }

            match receiver.await {
                Ok(None) => None,
                Ok(Some(bytes)) => Some((bytes, id)),
                Err(canceled) => {
                    warn!("Stream sender was dropped before resolving: {canceled}");
                    // assume connection is closed
                    None
                }
            }
        })
    }

    async fn sending_stream(&self, stream: impl Stream<Item=Vec<u8>> + MaybeSend + MaybeSync + 'static) -> StreamID {
        let id: StreamID = self.stream_ids.next();
        self.outgoing_streams.insert(id, Box::pin(stream)).await;
        id
    }
}

#[derive(Debug, Error, Clone)]
enum SendError<ConnError> {
    #[error("Failed to send request: {0}")]
    Conn(ConnError),
    #[error("Failed to send message: connection is closed")]
    ConnClosed(#[from] Canceled)
}

/// A client which communicates with the connection, taking care of message formatting and state management
pub struct Client<Conn, F> {
    connection_manager: Rc<ConnectionManager<Conn>>,
    format: F
}

impl<Conn: Connection + MaybeSend + 'static, F> Client<Conn, F> {
    pub(crate) fn new(conn: Conn, format: F) -> (Self, impl Future<Output = Result<(), ConnectionError>>) {
        let client = Self {
            connection_manager: Rc::new(ConnectionManager::new(conn)),
            format,
        };
        let job = client.connection_manager.clone().start();
        (client, job)
    }
}

impl<Conn, F> Clone for Client<Conn, F>
where
    ConnectionManager<Conn>: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            connection_manager: self.connection_manager.clone(),
            format: self.format.clone(),
        }
    }
}

impl<Conn, F> Client<Conn, F>
where
    Conn: Connection + Sync + Send + 'static,
{
    /// Send a request and await a response
    ///
    /// # Errors
    /// Returns an error if there was a connection error or an error in handling the request
    pub async fn send<Response, Request>(&self, request: Request) -> Result<Response, ClientError<Conn::SendError>>
    where
        F: Format<Response, Request>
    {
        let request = self.format.write(request).map_err(ClientError::Serialise)?;
        let response = self.connection_manager.send(request).await??;
        let response = self.format.read(&response).map_err(ClientError::Deserialise)?;
        Ok(response)
    }

    /// Create a new stream, streaming values from the underlying connection
    #[allow(clippy::unused_async, reason = "Consistency and forward compatability")]
    pub async fn receiving_stream<I: Send + Sync>(&self, id: StreamID) -> impl Stream<Item=I>
    where
        F: ReadFormat<I>
    {
        self.connection_manager.receiving_stream(id).filter_map(async |bytes| {
            match self.format.read(&bytes) {
                Ok(item) => Some(item),
                Err(error) => {
                    warn!("Failed to deserialise stream item: {error}");
                    None
                }
            }
        })
    }

    /// Stream this stream's values to the underlying connection with the returned id
    pub async fn sending_stream<I: Send + Sync>(&self, stream: impl Stream<Item=I> + MaybeSend + MaybeSync + 'static) -> StreamID
    where
        F: WriteFormat<I>
    {
        let format = self.format.clone();
        self.connection_manager.sending_stream(stream.filter_map(move |item| {
            match format.write(item) {
                Ok(bytes) => ready(Some(bytes)),
                Err(error) => {
                    warn!("Failed to serialise stream item: {error}");
                    ready(None)
                }
            }
        })).await
    }
}

/// Failed to send request
#[derive(Debug, Error)]
pub enum ClientError<ConnError> {
    /// The response was for the wrong method
    #[error(transparent)]
    WrongResponse(#[from] WrongResponseType),
    /// Failed to serialise request
    #[error("Failed to serialise request")]
    Serialise(Box<dyn std::error::Error + Send>),
    /// Failed to deserialise response
    #[error("Failed to deserialise request")]
    Deserialise(Box<dyn std::error::Error + Send>),
    /// Connection error
    #[error("Failed to send request: {0}")]
    Conn(ConnError),
    /// Connection closed
    #[error("Failed to send message: connection is closed")]
    ConnClosed(Canceled),
    /// Handler error
    #[error("Failed to handle request: {0}")]
    Handle(#[from] HandleError),
}

impl<ConnError> From<SendError<ConnError>> for ClientError<ConnError> {
    fn from(err: SendError<ConnError>) -> Self {
        match err {
            SendError::Conn(error) => Self::Conn(error),
            SendError::ConnClosed(error) => Self::ConnClosed(error),
        }
    }
}

impl<E> MaybeWrongResponse for ClientError<E> {
    fn into_wrong_response(self) -> Result<WrongResponseType, Self> {
        if let Self::WrongResponse(err) = self {
            Ok(err)
        } else {
            Err(self)
        }
    }
}

/// Represents an underlying websocket connection
pub struct ChannelConnection {
    response_sender: Sender<Vec<u8>>,
    request_receiver: Receiver<Vec<u8>>
}

impl Connection for ChannelConnection {
    type SendError = async_channel::SendError<Vec<u8>>;
    type RecvError = async_channel::RecvError;

    async fn send(&self, value: Vec<u8>) -> Result<(), Self::SendError> {
        self.response_sender.send(value).await
    }

    async fn recv(&self) -> Result<Vec<u8>, Self::RecvError> {
        self.request_receiver.recv().await
    }
}
