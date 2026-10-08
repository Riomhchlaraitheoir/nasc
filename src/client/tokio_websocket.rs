//! Defines a websocket client

use crate::format::IsFormat;
use futures::{SinkExt, StreamExt};
use async_channel::{RecvError};
use futures::future::{select, Either};
use tokio_tungstenite::tungstenite::{ClientRequestBuilder, Error, Message};
use tokio_tungstenite::{connect_async};
use tracing::{error, info, info_span, Instrument};
use std::pin::pin;
use crate::{MaybeSend, MaybeSync};
use crate::client::{Client, ChannelConnection};

/// Create a new websocket transport layer to the given URL
///
/// # Errors
/// Returns an error if an error occurs while establishing a connection
#[allow(
    clippy::missing_panics_doc,
    reason = "panic in theoretically unreachable"
)]
pub async fn new_websocket_transport<Format: IsFormat, OnClose: FnOnce() + MaybeSend + MaybeSync + 'static>(
    url: impl AsRef<str>,
    format: Format,
    on_close: OnClose
) -> Result<Client<ChannelConnection, Format>, Error> {
    let (mut websocket, _) = connect_async(
        ClientRequestBuilder::new(url.as_ref().parse().expect("failed to parse url"))
            .with_sub_protocol(format.subprotocol()),
    )
    .await?;
    let (request_sender, request_receiver) = async_channel::bounded(10);
    let (response_sender, response_receiver) = async_channel::bounded(10);
    let conn = ChannelConnection { response_sender, request_receiver };
    let (client, job) = Client::new(conn, format);
    tokio::spawn(async move {
        // let mut response_receiver = pin!(response_receiver);
        loop {
            let next_request = websocket.next();
            let next_request = pin!(next_request);
            let next_response = response_receiver.recv();
            let next_response = pin!(next_response);
            match select(next_response, next_request).await {
                Either::Left((Err(error), _)) => {
                    let _: RecvError = error;
                    on_close();
                    break
                },
                Either::Left((Ok(bytes), _)) => {
                    let result = websocket.send(Message::binary(bytes)).await;
                    if let Err(error) = result {
                        error!("failed to send response, closing websocket: {error}");
                        break
                    }
                }
                Either::Right((None, _)) => return, // ended by client
                Either::Right((Some(request), _)) => {
                    match parse_message(request) {
                        None => {},
                        Some(Either::Left(bytes)) => {
                            if let Err(error) = request_sender.send(bytes).await {
                                error!("failed to send parsed request, closing websocket: {error}");
                                break
                            }
                        },
                        Some(Either::Right(message)) => {
                            if let Err(error) = websocket.send(message).await {
                                error!("failed to send message, closing websocket: {error}");
                                break
                            }
                        }
                    }
                }
            }
        }
    }.instrument(info_span!("websocket message parser")));
    tokio::spawn(job);
    Ok(client)
}

fn parse_message(result: Result<Message, Error>) -> Option<Either<Vec<u8>, Message>> {
    let message = match result {
        Ok(message) => message,
        Err(error) => {
            info!("Websocket disconnected with error: {error}");
            return None;
        }
    };
    match message {
        Message::Text(text) => {
            error!("Received unexpected text message: {text}");
            None
        },
        Message::Binary(bytes) => {
            Some(Either::Left(bytes.into()))
        },
        Message::Ping(bytes) => Some(Either::Right(Message::Pong(bytes))),
        Message::Pong(_) => None,
        Message::Close(frame) => {
            if let Some(frame) = frame {
                info!("Websocket connection closed, code: {}, reason: {}", frame.code, frame.reason);
            } else {
                info!("Websocket connection closed without frame");
            }
            Some(Either::Right(Message::Close(None)))
        }
        Message::Frame(_) => unreachable!("Should never receive a raw message")
    }
}
