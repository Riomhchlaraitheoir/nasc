//! Integration with axum web server, see [`handle_websocket`]

#[allow(unused_imports, reason = "only used if certain features are enabled")]
use crate::format;
use crate::format::FormatSet;
use crate::{assert_stream, IntoHandler, assert_sink};
use async_executor::Executor;
use axum::response::Response;
use std::fmt::Debug;
use std::sync::Arc;
use axum::extract::ws::WebSocket;
use futures::{Sink, SinkExt};
use thiserror::Error;
use tokio::{select, spawn};
#[allow(
    unused_imports,
    reason = "may be unused depending on features, not worth splitting behind toggles"
)]
use tracing::{Instrument, debug, error, info, info_span, warn};

use crate::connection::channel::ChannelConnection;
use crate::connection::message::Message;
use crate::connection::{new_server_conn, Connection};
use {
    axum::extract::{
        WebSocketUpgrade,
        ws::Message as WsMessage,
    },
    futures::StreamExt,
    std::pin::pin,
};
use crate::connection::sink_stream::SinkStreamConn;

/// Handle an incoming websocket connection
///
/// Handle this connection, waiting for requests and sending responses
/// The server must implement [`ServiceBookends`](super::ServiceBookends), this trait controls
/// if and when idle connections are killed
///
/// # Errors
/// May fail to handle the connection
pub async fn handle_websocket<R, F>(
    ws: WebSocketUpgrade,
    server: impl IntoHandler<R, F::Format>,
    formats: &F
) -> Result<Response, WebsocketError>
where
    F: FormatSet,
{
    let ws = ws.protocols(formats.subprotocols());
    let protocol = ws
        .selected_protocol()
        .ok_or(WebsocketError::UnsupportedFormat)?;
    let format = formats
        .for_subprotocol(protocol)
        .ok_or(WebsocketError::UnsupportedFormat)?;
    // TODO: version negotiation
    let handler = server.into_handler(format);
    Ok(ws.on_upgrade(move |websocket| {

        let websocket = websocket.filter_map(async |result| {
            match result {
                Ok(message) => {
                    if let WsMessage::Binary(bytes) = message {
                        Some(Ok(bytes.to_vec()))
                    } else {
                        None
                    }
                }
                Err(error) => Some(Err(error)),
            }
        });
        let websocket = websocket.with(async |bytes: Vec<u8>| {
            Result::<_, <WebSocket as Sink<WsMessage>>::Error>::Ok(WsMessage::Binary(bytes.into()))
        });

        let websocket = Box::pin(websocket);

        let conn = SinkStreamConn::new(websocket);

        let executor = Arc::new(Executor::new());

        for _ in 0..4 {
            let executor = executor.clone();
            spawn(async move {
                loop {
                    executor.tick().await;
                }
            });
        }

        async {
            let driver = new_server_conn(handler).await;
            if let Err(error) = conn.run(driver, executor).await {
                error!("Websocket connection error: {error}");
            }
        }
    }))
}

#[derive(Debug, Error)]
/// An Error which may occur when accepting websocket connections
pub enum WebsocketError {
    #[error("Unsupported format")]
    /// The websocket protocol specifies an unsupported format
    UnsupportedFormat,
}

