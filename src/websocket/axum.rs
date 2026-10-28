//! Integration with axum web server, see [`handle_websocket`]

use crate::PROTOCOL_VERSION;
use crate::{IntoHandler, PROTOCOL_NAME};
#[allow(unused_imports, reason = "only used if certain features are enabled")]
use crate::format;
use crate::format::FormatSet;
use axum::extract::ws::WebSocket;
use axum::http::header::ToStrError;
use axum::response::Response;
use futures::{Sink, SinkExt};
use std::fmt::Debug;
use std::ops::ControlFlow;
use axum::http::HeaderValue;
use thiserror::Error;
#[allow(
    unused_imports,
    reason = "may be unused depending on features, not worth splitting behind toggles"
)]
use tracing::{Instrument, debug, error, info, info_span, warn};

use crate::connection::sink_stream::SinkStreamConn;
use crate::connection::{Connection, ConnectionOptions, new_server_conn};
use crate::runtime::TokioRuntime;
use {
    axum::extract::{WebSocketUpgrade, ws::Message as WsMessage},
    futures::StreamExt,
};

/// Handle an incoming websocket connection
///
/// # Errors
/// May reject the connection
pub fn handle_websocket<R, F>(
    mut ws: WebSocketUpgrade,
    server: impl IntoHandler<R, F::Format>,
    formats: &F,
    options: ConnectionOptions,
) -> Result<Response, WebsocketError>
where
    F: FormatSet + Sync,
{
    let selected = ws.requested_protocols()
        .cloned()
        .map(|header| parse_protocol(header, formats))
        .try_fold(None, |acc, result| {
            if result.is_ok() {
                ControlFlow::Break(result)
            } else {
                ControlFlow::Continue(acc.or(Some(result)))
            }
        });
    let (ControlFlow::Break(result) | ControlFlow::Continue(Some(result))) = selected else {
        return Err(WebsocketError::UnsupportedProtocol) // no protocol requests
    };
    let (header, format) = result?;
    ws.set_selected_protocol(header);

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

        async {
            let driver = new_server_conn(handler, options, TokioRuntime).await;
            if let Err(error) = conn.run(driver).await {
                error!("Websocket connection error: {error}");
            }
        }
    }))
}

fn parse_protocol<F: FormatSet>(header: HeaderValue, formats: &F) -> Result<(HeaderValue, F::Format), WebsocketError> {
    let protocol = header.to_str()?;
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
    let format = formats.for_subprotocol(format).ok_or(WebsocketError::UnsupportedFormat)?;
    Ok((header, format))
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
