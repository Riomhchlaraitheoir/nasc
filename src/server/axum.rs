//! Integration with axum web server, see [`handle_websocket`]

#[allow(unused_imports, reason = "only used if certain features are enabled")]
use crate::format;
use crate::format::{Format, FormatSet};
use crate::{Executor, Handler, Rc, Rpc, RpcWithServer};
use axum::response::Response;
use std::fmt::Debug;
use thiserror::Error;
use tokio::{select, spawn};
#[allow(
    unused_imports,
    reason = "may be unused depending on features, not worth splitting behind toggles"
)]
use tracing::{Instrument, debug, error, info, info_span, warn};

use crate::server::{ServerHooks, Server};
use {
    axum::extract::{
        WebSocketUpgrade,
        ws::Message,
    },
    futures::{
        FutureExt
        ,
        StreamExt},
    std::pin::pin,
};

/// Handle an incoming websocket connection
///
/// Handle this connection, waiting for requests and sending responses
/// The server must implement [`ServiceBookends`](super::ServiceBookends), this trait controls
/// if and when idle connections are killed
///
/// # Errors
/// May fail to handle the connection
pub fn handle_websocket<R, Service, F>(
    ws: WebSocketUpgrade,
    server: Service,
    formats: &F
) -> Result<Response, WebsocketError>
where
    R: RpcWithServer<Service>,
    F: FormatSet,
    <R as RpcWithServer<Service>>::Handler<F::Format>: Handler<F::Format, Rpc = R, Server = Service> + Sync + 'static,
    <R as Rpc>::Request: Send,
    <R as Rpc>::Response: Send + Sync,
    F::Format: Format<<R as Rpc>::Request, <R as Rpc>::Response> + 'static,
    Service: ServerHooks<R> + Send + Sync
{
    let handler = R::handler(server);
    let ws = ws.protocols(formats.subprotocols());
    let protocol = ws
        .selected_protocol()
        .ok_or(WebsocketError::UnsupportedFormat)?;
    let format = formats
        .for_subprotocol(protocol)
        .ok_or(WebsocketError::UnsupportedFormat)?;
    Ok(ws.on_upgrade(move |websocket| {
        // let (sink_lock, stream_lock) = BiLock::new(websocket);
        // let stream = websocket_stream(stream_lock);
        // let mut stream = pin!(stream);
        //
        // let sink = websocket_sink(sink_lock);
        // let mut sink = pin!(sink);
        //
        let (request_sender, request_receiver) = async_channel::bounded(10);
        let (response_sender, response_receiver) = async_channel::bounded(10);

        spawn(async move {
            let mut websocket = pin!(websocket);
            'conn: loop {
                select! {
                    next = websocket.next() => {
                        let Some(next) = next else {
                            break 'conn
                        };
                        let message = match next {
                            Ok(message) => message,
                            Err(error) => {
                                info!("Websocket disconnected with error: {error}");
                                break 'conn;
                            }
                        };
                        let mut should_close = false;
                        let result = match message {
                            Message::Text(_) => {
                                websocket
                                    .send(Message::Text("text frames not supported".into()))
                                    .await
                            }
                            Message::Binary(bytes) => {
                                let Ok(()) = request_sender.send(bytes.to_vec()).await else {
                                    break 'conn;
                                };
                                continue 'conn
                            }
                            Message::Ping(bytes) => websocket.send(Message::Pong(bytes)).await,
                            Message::Pong(_) => Ok(()),
                            Message::Close(frame) => {
                                if let Some(frame) = frame {
                                    info!(
                                        "Websocket connection closed, code: {}, reason: {}",
                                        frame.code, frame.reason
                                    );
                                } else {
                                    info!("Websocket connection closed without frame");
                                }
                                should_close = true;
                                websocket.send(Message::Close(None)).await
                            }
                        };
                        if let Err(error) = result {
                            error!("Websocket connection error: {error}");
                            break 'conn;
                        }
                        if should_close {
                            break 'conn;
                        }
                    }
                    response = response_receiver.recv() => {
                        let Ok(response) = response else {
                            break 'conn
                        };
                        let result = {
                            debug!("Sending response: {response:?}");
                            websocket.send(Message::binary(response)).await
                        };
                        if let Err(error) = result {
                            warn!("Failed to send response: {error}");
                            break 'conn;
                        }
                    }
                }
            }
        });

        let executor = Rc::new(Executor::new());

        for _ in 0..4 {
            let executor = executor.clone();
            spawn(async move {
                loop {
                    executor.tick().await;
                }
            });
        }

        let server =Server::new(handler, format, request_receiver, response_sender, executor);
        Box::leak(Box::new(server))
            .start()
            .map(|result| {
            if let Err(error) = result {
                error!("Error occurred on websocket connection: {error}");
            }
        })
    }))
}

#[derive(Debug, Error)]
/// An Error which may occur when accepting websocket connections
pub enum WebsocketError {
    #[error("Unsupported format")]
    /// The websocket protocol specifies an unsupported format
    UnsupportedFormat,
}

