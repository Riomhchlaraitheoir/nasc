//! Defines a websocket client

use crate::format::IsFormat;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::{Message, WebSocketError};
use std::pin::pin;
use async_channel::RecvError;
use futures::future::{select, Either};
use thiserror::Error;
use tracing::{debug, error, info_span, Instrument, info};
use wasm_bindgen_futures::spawn_local;
use crate::client::{ChannelConnection, Client};

/// Error from websocket
#[derive(Debug, Error)]
pub enum WebsocketError {
    /// Failed to connect to websocket
    #[error("Failed to connect: {0}")]
    ConnectionError(String),
}

/// Create a new websocket transport layer to the given URL
///
/// # Errors
/// Returns an error if an error occurs while establishing a connection
#[expect(clippy::unused_async, reason = "might be needed in the future")]
pub async fn new_websocket_transport<Format: IsFormat, OnClose: FnOnce() + 'static>(
    url: impl AsRef<str>,
    format: Format,
    on_close: OnClose,
) -> Result<Client<ChannelConnection, Format>, WebsocketError> {
    debug!("Connecting websocket to {}", url.as_ref());
    let mut websocket = WebSocket::open_with_protocol(url.as_ref(), format.subprotocol())
        .map_err(|e| WebsocketError::ConnectionError(e.message))?;
    let (request_sender, request_receiver) = async_channel::bounded(10);
    let (response_sender, response_receiver) = async_channel::bounded(10);
    let conn = ChannelConnection { response_sender, request_receiver };
    let (client, job) = Client::new(conn, format);

    spawn_local(async move {
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
                    let result = websocket.send(Message::Bytes(bytes)).await;
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
    spawn_local(async {
        if let Err(error) = job.await {
            error!("Connection failed: {error}");
        }
    });
    Ok(client)
}

fn parse_message(result: Result<Message, WebSocketError>) -> Option<Either<Vec<u8>, Message>> {
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
        Message::Bytes(bytes) => {
            Some(Either::Left(bytes))
        },
    }
}
