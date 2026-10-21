//! Defines a websocket client

use crate::connection::channel::ChannelConnection;
use crate::connection::client::Client;
use crate::connection::message::Message;
use crate::connection::new_client_conn;
use crate::format::IsFormat;
use crate::spawner::BrowserSpawner;
use async_channel::RecvError;
use futures::channel::oneshot;
use futures::future::Either;
use futures::{FutureExt, SinkExt, StreamExt, select};
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::{Message as WsMessage, WebSocketError};
use std::pin::pin;
use thiserror::Error;
use tracing::{Instrument, debug, error, info, info_span};
use wasm_bindgen_futures::spawn_local;

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
pub async fn new_websocket_transport<Format: IsFormat, OnClose: FnOnce() + 'static>(
    url: impl AsRef<str>,
    format: Format,
    on_close: OnClose,
) -> Result<Client, WebsocketError> {
    debug!("Connecting websocket to {}", url.as_ref());
    let websocket = WebSocket::open_with_protocol(url.as_ref(), format.subprotocol())
        .map_err(|e| WebsocketError::ConnectionError(e.message))?;
    let (conn, request_sender, response_receiver) = ChannelConnection::pair();

    let client_ready = oneshot::channel();
    spawn_local(async {
        let (client, driver) = new_client_conn().await;
        let Ok(()) = client_ready.0.send(client) else {
            return
        };
        if let Err(error) = driver.run(conn, BrowserSpawner).await {
            error!("connection error: {}", error);
        }
    });


    spawn_local(async move {
        let (mut outgoing, mut incoming) = websocket.split();
        let mut next_request = pin!(incoming.next().fuse());
        let mut next_response = pin!(response_receiver.recv().fuse());
        loop {
            select! {
                result = next_request => {
                    match result {
                        Some(request) => {
                            match parse_message(request) {
                                None => {},
                                Some(Either::Left(bytes)) => {
                                    let message = match Message::deserialise(&bytes) {
                                        Ok(message) => message,
                                        Err(error) => {
                                            error!("Failed to parse message: {error}");
                                            let message = WsMessage::Bytes(Message::ConnectionError(error.into()).serialise());
                                            if let Err(error) = outgoing.send(message).await {
                                                error!("failed to send message, closing websocket: {error}");
                                                break
                                            }
                                            continue
                                        }
                                    };
                                    if let Err(error) = request_sender.send(message).await {
                                        error!("failed to send parsed request, closing websocket: {error}");
                                        break
                                    }
                                },
                                Some(Either::Right(message)) => {
                                    if let Err(error) = outgoing.send(message).await {
                                        error!("failed to send message, closing websocket: {error}");
                                        break
                                    }
                                }
                            }
                        }
                        None => {
                            return
                        }
                    }
                },
                result = next_response => {
                    match result {
                        Ok(message) => {
                            let bytes = message.serialise();
                            let result = outgoing.send(WsMessage::Bytes(bytes)).await;
                            if let Err(error) = result {
                                error!("failed to send response, closing websocket: {error}");
                                break
                            }

                        }
                        Err(error) => {
                            let _: RecvError = error;
                            on_close();
                            break
                        }
                    }
                },
            }
        }
    }.instrument(info_span!("websocket message parser")));
    Ok(client_ready.1.await.expect("client failed to start"))
}

fn parse_message(result: Result<WsMessage, WebSocketError>) -> Option<Either<Vec<u8>, WsMessage>> {
    let message = match result {
        Ok(message) => message,
        Err(error) => {
            info!("Websocket disconnected with error: {error}");
            return None;
        }
    };
    match message {
        WsMessage::Text(text) => {
            error!("Received unexpected text message: {text}");
            None
        },
        WsMessage::Bytes(bytes) => {
            Some(Either::Left(bytes))
        },
    }
}
