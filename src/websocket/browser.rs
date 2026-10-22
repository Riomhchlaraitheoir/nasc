//! Defines a websocket client

use crate::__macros::imports::Sink;
use crate::connection::client::Client;
use crate::connection::sink_stream::SinkStreamConn;
use crate::connection::{new_client_conn, Connection, ConnectionOptions};
use crate::format::IsFormat;
use crate::{PROTOCOL_VERSION, PROTOCOL_NAME};
use crate::runtime::BrowserRuntime;
use futures::channel::oneshot;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message as WsMessage;
use thiserror::Error;
use tracing::{debug, error};
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
    options: ConnectionOptions
) -> Result<Client<BrowserRuntime>, WebsocketError> {
    debug!("Connecting websocket to {}", url.as_ref());
    let protocol = format!("{PROTOCOL_NAME}.{PROTOCOL_VERSION}.{}", format.subprotocol());
    let websocket = WebSocket::open_with_protocol(url.as_ref(), &protocol)
        .map_err(|e| WebsocketError::ConnectionError(e.message))?;

    let websocket = websocket.filter_map(async |result| {
        match result {
            Ok(message) => {
                if let WsMessage::Bytes(bytes) = message {
                    Some(Ok(bytes))
                } else {
                    None
                }
            }
            Err(error) => Some(Err(error)),
        }
    });
    let websocket = websocket.with(async |bytes: Vec<u8>| {
        Result::<_, <WebSocket as Sink<WsMessage>>::Error>::Ok(WsMessage::Bytes(bytes))
    });
    let websocket = Box::pin(websocket);
    let conn = SinkStreamConn::new_combined(websocket);

    let client_ready = oneshot::channel();
    spawn_local(async {
        let (client, driver) = new_client_conn(options, BrowserRuntime).await;
        let Ok(()) = client_ready.0.send(client) else {
            return
        };
        if let Err(error) = conn.run(driver).await {
            error!("connection error: {}", error);
        }
        on_close();
    });

    Ok(client_ready.1.await.expect("client failed to start"))
}

