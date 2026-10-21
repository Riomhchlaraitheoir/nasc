use std::fmt::Display;
use std::pin::pin;
use futures::{select, FutureExt, Sink, SinkExt, Stream, StreamExt};
use tracing::{debug, info, warn};
use crate::connection::{Connection, ConnectionDriver};
use crate::connection::message::Message;
use crate::spawner::Spawner;

pub struct SinkStreamConn<S> {
    sink_stream: S,
}

impl<S> SinkStreamConn<S> where Self: Connection {
    pub fn new(sink_stream: S) -> Self {
        Self { sink_stream }
    }
}

impl<S, RecvError> Connection for SinkStreamConn<S>
where
    S: Sink<Vec<u8>, Error: Display>,
    S: Stream<Item = Result<Vec<u8>, RecvError>>,
    S: Send + Unpin + 'static,
    RecvError: Display + Send + 'static,
{
    async fn run(mut self, driver: ConnectionDriver, spawner: impl Spawner + 'static) -> Result<(), impl Display> {
        let (mut sink, mut stream) = self.sink_stream.split();
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
                        let mut should_close = false;

                        match Message::deserialise(&bytes) {
                            Ok(message) => {
                                if let Err(error) = driver.consume_incoming_message(message, &spawner).await {
                                    break Err(format!("Connection error: {error}"));
                                };
                            },
                            Err(error) => {
                                let error_msg = error.to_string();
                                let bytes = Message::from(error).serialise();
                                let Ok(()) = sink.send(bytes).await else {
                                    break Err(format!("Send error: {error_msg}"));
                                };
                            }
                        }
                        continue
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