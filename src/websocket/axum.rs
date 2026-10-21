//! Integration with axum web server, see [`handle_websocket`]

use crate::{IntoHandler, PROTOCOL_NAME};
use crate::PROTOCOL_VERSION;
#[allow(unused_imports, reason = "only used if certain features are enabled")]
use crate::format;
use crate::format::FormatSet;
use async_executor::Executor;
use axum::extract::ws::WebSocket;
use axum::response::Response;
use futures::{Sink, SinkExt};
use std::fmt::Debug;
use std::ops::ControlFlow;
use std::sync::Arc;
use axum::http::header::ToStrError;
use thiserror::Error;
use tokio::spawn;
#[allow(
    unused_imports,
    reason = "may be unused depending on features, not worth splitting behind toggles"
)]
use tracing::{Instrument, debug, error, info, info_span, warn};

use crate::connection::sink_stream::SinkStreamConn;
use crate::connection::{Connection, new_server_conn};
use {
    axum::extract::{WebSocketUpgrade, ws::Message as WsMessage},
    futures::StreamExt,
};

/// Handle an incoming websocket connection
///
/// Handle this connection, waiting for requests and sending responses
/// The server must implement [`ServiceBookends`](super::ServiceBookends), this trait controls
/// if and when idle connections are killed
///
/// # Errors
/// May fail to handle the connection
pub fn handle_websocket<R, F>(
    ws: WebSocketUpgrade,
    server: impl IntoHandler<R, F::Format> + Send,
    formats: &F,
) -> Result<Response, WebsocketError>
where
    F: FormatSet + Sync,
{
    let selected = ws.requested_protocols().map(|header| {
        header.to_str()
    }).collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|header| parse_protocol(header, formats))
        .try_fold(None, |acc: Option<Result<F::Format, WebsocketError>>, result| {
            if result.is_ok() {
                ControlFlow::Break(result)
            } else {
                ControlFlow::Continue(acc.or(Some(result)))
            }
        });
    let (ControlFlow::Break(result) | ControlFlow::Continue(Some(result))) = selected else {
        return Err(WebsocketError::UnsupportedProtocol) // no protocol requests
    };
    let format = result?;

    let handler = server.into_handler(format);
    Ok(ws.on_upgrade(move |websocket| {
        let websocket = websocket.filter_map(async |result| match result {
            Ok(message) => {
                if let WsMessage::Binary(bytes) = message {
                    Some(Ok(bytes.to_vec()))
                } else {
                    None
                }
            }
            Err(error) => Some(Err(error)),
        });
        let websocket = websocket.with(async |bytes: Vec<u8>| {
            Result::<_, <WebSocket as Sink<WsMessage>>::Error>::Ok(WsMessage::Binary(bytes.into()))
        });

        let websocket = Box::pin(websocket);

        let conn = SinkStreamConn::new_combined(websocket);

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

fn parse_protocol<F: FormatSet>(protocol: &str, formats: &F) -> Result<F::Format, WebsocketError> {
    let mut parts = protocol.split('.');
    let protocol = parts.next().ok_or(WebsocketError::UnsupportedProtocol)?;
    if protocol != PROTOCOL_NAME {
        return Err(WebsocketError::UnsupportedProtocol);
    }
    let version = parts.next().ok_or(WebsocketError::UnsupportedProtocol)?;
    if version != PROTOCOL_VERSION {
        return Err(WebsocketError::UnsupportedVersion);
    }
    let format = parts.next().ok_or(WebsocketError::UnsupportedProtocol)?;
    formats.for_subprotocol(format).ok_or(WebsocketError::UnsupportedFormat)
}

#[derive(Debug, Error)]
/// An Error which may occur when accepting websocket connections
pub enum WebsocketError {
    #[error("Unsupported protocol")]
    /// Unsupported protocol
    UnsupportedProtocol,
    #[error("Unsupported version")]
    /// Unsupported version
    UnsupportedVersion,
    #[error("Unsupported format")]
    /// The websocket protocol specifies an unsupported format
    UnsupportedFormat,
    /// Failed to decode requested protocol
    #[error("Failed to decode requested protocol: {0}")]
    ToStr(#[from] ToStrError),
}
