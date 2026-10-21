//! Contains modules for individual server implementations

use futures::FutureExt;
use crate::format::{Format, ReadFormat, WriteFormat};
use crate::message::{ConnectionError, ConnectionMessage, RequestID, StreamID};
use crate::{BoxStream, ConnectionController, Executor, Handler, Send, Sync, Rc, Rpc, Series};
use async_channel::{Receiver, Sender};
use futures::channel::oneshot;
use futures::stream::unfold;
use futures::{Stream, StreamExt};
use tracing::{error, warn};
use crate::shard_map::ShardMap;

// /// Helpers for serving a service from an axum server
#[cfg(feature = "axum")]
pub mod axum;

/// Defines hooks for a connection-based service (i.e. WebSocket), can be used:
/// - to kill idle connections
/// - to log requests and responses
/// - to track metrics such as request counts or response delays
pub trait ServerHooks<R: Rpc> {
    /// Wait for the next request or timeout
    ///
    /// The request future will yield when the next request arrives, or None if the client closes
    /// the connection
    ///
    /// This function should not be used to inspect the request, use other hooks for that
    ///
    /// To cancel waiting and close the connection instead, simply return `None`
    ///
    /// By default, it will simply pass the given future through, meaning no timeouts, rate limiting, etc.
    fn await_next<T>(
        &self,
        request: impl Future<Output = Option<T>> + Send,
    ) -> impl Future<Output = Option<T>> + Send {
        request
    }

    /// Called after receiving a request, but before it is processed
    ///
    /// - `request_id` is the identifier of the request
    fn request(&self, request_id: RequestID, request: &<R as Rpc>::Request) {
        let _ = (request_id, request);
    }

    /// Called after processing a request, but before sending the response
    ///
    /// - `request_id` is the identifier of the request
    /// - `is_streaming` indicates if this is a streaming response (part of an `impl Stream`)
    fn response(&self, request_id: RequestID, response: &<R as Rpc>::Response) {
        let _ = (request_id, response);
    }
}

/// This type manages a connection (either client-side or server-side), manages all incoming and
/// outgoing streams and allows higher layers to focus on request/response without dealing without
/// the streams
///
/// NOTE: This will only work as long as [receive](Self::receive) is regularly polled
struct Server<'a, H: Handler<F>, F> {
    handler: H,
    format: F,
    incoming: Receiver<Vec<u8>>,
    outgoing: Sender<Vec<u8>>,
    executor: Rc<Executor<'a>>,
    outgoing_streams: ShardMap<StreamID, BoxStream<'a, Vec<u8>>>,
    polled_incoming_streams: ShardMap<StreamID, oneshot::Sender<Option<Vec<u8>>>>,
    stream_ids: Rc<Series<StreamID>>,
}

impl<'a, H, F> Server<'a, H, F>
where
    H: Handler<F> + Sync + 'a,
    H::Server: ServerHooks<H::Rpc>,
    F: Format<<H::Rpc as Rpc>::Request, <H::Rpc as Rpc>::Response>,
{
    pub fn new(
        handler: H,
        format: F,
        incoming: Receiver<Vec<u8>>,
        outgoing: Sender<Vec<u8>>,
        executor: Rc<Executor<'a>>,
    ) -> Self {
        Self {
            handler,
            format,
            incoming,
            outgoing,
            executor,
            outgoing_streams: ShardMap::new(),
            polled_incoming_streams: ShardMap::new(),
            stream_ids: Rc::default(),
        }
    }

    pub(crate) async fn start(&'a self) -> Result<(), ConnectionError> {
        loop {
            let next = self.incoming.recv().map(Result::ok);
            let Some(next) = self.handler.server().await_next(next).await else {
                break Ok(());
            };
            let next = ConnectionMessage::parse(&next);
            let (request_id, request) = match next {
                ConnectionMessage::ConnectionError(error) => {
                    break Err(error);
                }
                ConnectionMessage::Payload {
                    request_id,
                    payload,
                } => (request_id, payload),
                ConnectionMessage::HandleError { .. } => {
                    error!("Receiver handle error in server");
                    continue;
                }
                ConnectionMessage::StreamItem { stream_id, item } => {
                    if let Some(sender) = self.polled_incoming_streams.remove(&stream_id).await {
                        // error indicates handling function dropped stream, no longer interested
                        let _: Result<_, Option<Vec<u8>>> = sender.send(item);
                    } else {
                        warn!("stream {stream_id} was not polled");
                    }
                    continue;
                }
                ConnectionMessage::StreamPoll { stream_id } => {
                    self.executor
                        .spawn(async move {
                            let item = if let Some(mut stream) =
                                self.outgoing_streams.remove(&stream_id).await
                            {
                                let item = stream.next().await;
                                if item.is_some() {
                                    self.outgoing_streams.insert(stream_id, stream).await;
                                }
                                item
                            } else {
                                warn!("Stream {stream_id} not found");
                                None
                            };
                            let result = self
                                .outgoing
                                .send(ConnectionMessage::StreamItem { stream_id, item }.format())
                                .await;
                            if let Err(error) = result {
                                error!("Error while sending message: {error:?}");
                            }
                        })
                        .detach();
                    continue;
                }
            };
            let request = match self.format.read(&request) {
                Ok(request) => request,
                Err(error) => {
                    error!("Error while reading request: {error:?}");
                    continue;
                }
            };
            self.handler.server().request(request_id, &request);
            let response = self.handler.handle(request, self).await;
            self.handler.server().response(request_id, &response);
            let response = match self.format.write(response) {
                Ok(response) => response,
                Err(error) => {
                    error!("Error while writing response: {error:?}");
                    continue;
                }
            };
            let Ok(()) = self
                .outgoing
                .send(
                    ConnectionMessage::Payload {
                        request_id,
                        payload: response,
                    }
                    .format(),
                )
                .await
            else {
                break Err(ConnectionError::ResponseChannelClosed);
            };
        }
    }

    fn raw_receiving_stream(&self, id: StreamID) -> impl Stream<Item = Vec<u8>> + Send {
        unfold(id, async |id| {
            let (sender, receiver) = oneshot::channel();
            self.polled_incoming_streams.insert(id, sender).await;

            let result: Result<(), _> = self
                .outgoing
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

    async fn raw_sending_stream(
        &self,
        stream: impl Stream<Item = Vec<u8>> + Send + 'a,
    ) -> StreamID {
        let id: StreamID = self.stream_ids.next();
        self.outgoing_streams.insert(id, Box::pin(stream)).await;
        id
    }
}

impl<'a, H, F> ConnectionController<'a, F> for Server<'a, H, F>
where
    H: Handler<F> + Sync + 'a,
    H::Server: ServerHooks<H::Rpc>,
    F: Format<<H::Rpc as Rpc>::Request, <H::Rpc as Rpc>::Response>,
{
    async fn receiving_stream<I: Send>(
        &self,
        id: StreamID,
    ) -> impl Stream<Item = I> + Send
    where
        F: ReadFormat<I>,
    {
        self.raw_receiving_stream(id)
            .filter_map(async |bytes| match self.format.read(&bytes) {
                Ok(response) => Some(response),
                Err(error) => {
                    error!("Error while reading stream item: {error:?}");
                    None
                }
            })
    }

    async fn sending_stream<I>(
        &self,
        stream: impl Stream<Item = I> + Send + 'a,
    ) -> StreamID
    where
        F: WriteFormat<I>,
        I: Send + Sync + 'static,
    {
        let format = self.format.clone();
        self.raw_sending_stream(
            stream.filter_map(move |item| {
                let format = format.clone();
                async move {
                    match format.write(item) {
                        Ok(bytes) => Some(bytes),
                        Err(error) => {
                            error!("Error while writing stream item: {error:?}");
                            None
                        }
                    }
                }
            }),
        )
        .await
    }
}
