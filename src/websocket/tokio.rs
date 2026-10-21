//! Defines a websocket client

use crate::connection::message::Message;
use crate::connection::{channel::ChannelConnection, client::Client, new_client_conn, Connection};
use crate::format::IsFormat;
use crate::spawner::TokioSpawner;
use async_channel::RecvError;
use futures::future::Either;
use futures::{FutureExt, SinkExt, StreamExt, select};
use std::pin::pin;
use axum::extract::ws::WebSocket;
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::tungstenite::{ClientRequestBuilder, Error, Message as WsMessage};
use tracing::{Instrument, error, info, info_span};
use crate::__macros::imports::Sink;
use crate::connection::sink_stream::SinkStreamConn;

/// Create a new websocket transport layer to the given URL
pub async fn new_websocket_transport<Format: IsFormat, OnClose: FnOnce() + Send + Sync + 'static>(
    url: impl AsRef<str>,
    format: Format,
    on_close: OnClose
) -> Result<Client, Error> {
    let (websocket, _) = connect_async(
        ClientRequestBuilder::new(url.as_ref().parse().expect("failed to parse url"))
            .with_sub_protocol(format.subprotocol()),
    )
    .await?;

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
        Result::<_, <WebSocketStream<MaybeTlsStream<TcpStream>> as Sink<WsMessage>>::Error>::Ok(WsMessage::Binary(bytes.into()))
    });
    let websocket = Box::pin(websocket);
    let conn = SinkStreamConn::new(websocket);


    let client_ready = oneshot::channel();
    tokio::spawn(async {
        let (client, driver) = new_client_conn().await;
        let Ok(()) = client_ready.0.send(client) else {
            return
        };
        if let Err(error) = conn.run(driver, TokioSpawner).await {
            error!("connection error: {}", error);
        }
    });

    Ok(client_ready.1.await.expect("client failed to start"))
}
