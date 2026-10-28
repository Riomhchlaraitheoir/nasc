//! Support for using a sink/stream combo as a connection, sink/stream should operate on `Vec<u8>`
//! and ensure that each `Vec<u8>` represents a single message

use std::fmt::Display;
use std::pin::pin;
use futures::{select, FutureExt, Sink, SinkExt, Stream, StreamExt};
use futures::stream::{SplitSink, SplitStream};
use tracing::{debug, warn};
use crate::connection::{Connection, ConnectionDriver};
use crate::connection::message::Message;

/// A Sink/Stream based connection, sending and receiving serialised binary messages
pub struct SinkStreamConn<Sink, Stream> {
    sink: Sink,
    stream: Stream,
}

impl<S, RecvError> SinkStreamConn<SplitSink<S, Vec<u8>>, SplitStream<S>>
where
    S: Sink<Vec<u8>, Error: Display> + Stream<Item = Result<Vec<u8>, RecvError>> + Unpin + 'static,
    RecvError: Display + 'static, {
    /// New [`SinkStreamConn`] from a value which is both a stream and a sink
    pub fn new_combined(sink_stream: S) -> Self {
        let (sink, stream) = sink_stream.split();
        Self { sink, stream }
    }
}

impl<Sink, Stream> SinkStreamConn<Sink, Stream> where Self: Connection {
    /// New [`SinkStreamConn`] from a separate sink and stream
    pub fn new(sink: Sink, stream: Stream) -> Self {
        Self { sink, stream }
    }
}

impl<Out, In, RecvError> Connection for SinkStreamConn<Out, In>
where
    Out: Sink<Vec<u8>, Error: Display> + Unpin + 'static,
    In: Stream<Item = Result<Vec<u8>, RecvError>> + Unpin + 'static,
    RecvError: Display + 'static,
{
    async fn run(self, driver: ConnectionDriver) -> Result<(), impl Display> {
        let Self { mut sink, mut stream } = self;
        let mut next_out = pin!(driver.next_outgoing_message().fuse());
        loop {
            select! {
                    next = stream.next().fuse() => {
                        let Some(next) = next else {
                            break Ok(())
                        };
                        let bytes = match next {
                            Ok(message) => message,
                            Err(error) => {
                                break Err(format!("Error from stream: {error}"));
                            }
                        };

                        match Message::deserialise(&bytes) {
                            Ok(message) => {
                                if let Err(error) = driver.consume_incoming_message(message).await {
                                    break Err(format!("Connection error: {error}"));
                                }
                            },
                            Err(error) => {
                                let error_msg = error.to_string();
                                let bytes = Message::from(error).serialise();
                                let Ok(()) = sink.send(bytes).await else {
                                    break Err(format!("Send error: {error_msg}"));
                                };
                            }
                        }
                    }
                    response = next_out => {
                        next_out.set(driver.next_outgoing_message().fuse());
                        let Some(response) = response else {
                            break Ok(())
                        };
                        let result = {
                            debug!("Sending response: {response:?}");
                            let response = response.serialise();
                            sink.send(response).await
                        };
                        if let Err(error) = result {
                            warn!("Failed to send response: {error}");
                            break Err(format!("Send error: {error}"));
                        }
                    }
                }
        }
    }
}