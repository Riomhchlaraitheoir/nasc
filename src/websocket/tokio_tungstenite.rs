//! Defines a websocket client

use crate::connection::{new_client_conn, Connection, ConnectionOptions};
use crate::format::Format;
use crate::{Service, PROTOCOL_NAME, PROTOCOL_VERSION};
use crate::runtime::TokioRuntime;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::tungstenite::{ClientRequestBuilder, Error, Message as WsMessage};
use tracing::error;
use crate::__macros::imports::Sink;
use crate::connection::sink_stream::SinkStreamConn;

/// Create a new websocket transport layer to the given URL
pub async fn new_websocket_transport<S: Service, F: Format>(
    url: impl AsRef<str>,
    format: F,
    on_close: impl FnOnce() + Send + Sync + 'static,
    options: ConnectionOptions
) -> Result<S::Client<F>, Error> {
    let protocol = format!("{PROTOCOL_NAME}.{PROTOCOL_VERSION}.{}", format.subprotocol());
    let (websocket, _) = connect_async(
        ClientRequestBuilder::new(url.as_ref().parse().expect("failed to parse url"))
            .with_sub_protocol(protocol),
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
    let conn = SinkStreamConn::new_combined(websocket);


    let client_ready = oneshot::channel();
    tokio::spawn(async {
        let (client, driver) = new_client_conn(options, TokioRuntime).await;
        let Ok(()) = client_ready.0.send(client) else {
            return
        };
        if let Err(error) = conn.run(driver).await {
            error!("connection error: {}", error);
        }
        on_close();
    });

    let client = client_ready.1.await.expect("client failed to start");
    Ok(S::client(client, format))
}
