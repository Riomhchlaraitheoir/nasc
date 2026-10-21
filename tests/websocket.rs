use async_executor::Executor;
use axum::extract::{FromRequestParts, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use futures::stream::iter;
use futures::{Stream, StreamExt};
use macros::rpc;
use std::pin::pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::spawn;
use tokio::sync::{RwLock, oneshot};
use tokio::time::{sleep, timeout};
use tracing::log::Level;
use tracing::{Instrument, debug, info, info_span};
use trait_rpc::format::json::Json;
use trait_rpc::websocket::axum::{WebsocketError, handle_websocket};
use trait_rpc::websocket::tokio::new_websocket_transport;

#[rpc]
trait Service {
    fn simple_get() -> String;
    fn simple_set(value: u64);
    fn stream_out() -> impl Stream<Item = u64>;
    fn stream_in(input: impl Stream<Item = u64>) -> u64;
}

#[derive(Default)]
struct ServerState {
    set_value: u64,
}

#[derive(FromRequestParts)]
struct ServiceImpl {
    state: State<Arc<RwLock<ServerState>>>,
}

impl ServiceServer for ServiceImpl {
    async fn simple_get(&self) -> String {
        "hello world".to_owned()
    }

    async fn simple_set(&self, value: u64) {
        self.state.write().await.set_value = value;
    }

    async fn stream_out(&self) -> impl Stream<Item = u64> + Send + 'static {
        futures::stream::iter(0..100).map(|i| {
            debug!("Sending {i}");
            i
        })
    }

    async fn stream_in(&self, input: impl Stream<Item = u64> + Send) -> u64 {
        input.count().await as u64
    }
}

async fn run_server(
    state: Arc<RwLock<ServerState>>,
    shutdown_signal: impl Future<Output = ()> + Send + 'static,
) {
    let server = axum::Router::new()
        .route("/", get(handle))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9865")
        .await
        .unwrap();
    info!("Starting server");
    axum::serve::serve(listener, server)
        .with_graceful_shutdown(shutdown_signal)
        .await
        .unwrap();
}

async fn handle(server: ServiceImpl, websocket: WebSocketUpgrade) -> impl IntoResponse {
    let result = handle_websocket::<Service, _>(websocket, server, &Json);
    match result {
        Ok(response) => response,
        Err(
            WebsocketError::UnsupportedFormat
            | WebsocketError::UnsupportedProtocol
            | WebsocketError::UnsupportedVersion
            | WebsocketError::ToStr(_),
        ) => StatusCode::BAD_REQUEST.into_response(),
    }
}

#[tokio::test]
async fn websocket_test() {
    use futures::FutureExt;
    simple_logger::init_with_level(Level::Info).unwrap();
    println!("Starting test");

    let (stop_server, shutdown_signal) = oneshot::channel::<()>();
    let state = Arc::<RwLock<ServerState>>::default();
    let server = spawn(
        run_server(state.clone(), shutdown_signal.into_future().map(|_| ()))
            .instrument(info_span!("server")),
    );
    info!("Server running");
    sleep(Duration::from_secs(1)).await;
    let client = {
        let client = new_websocket_transport("ws://localhost:9865/", Json, || {})
            .await
            .expect("failed to create client");
        Service::client(client, Json)
    };

    let executor = Executor::new();
    let tests = [
        executor.spawn(test_simple_get(&client)),
        executor.spawn(test_simple_set(&client, &state)),
        executor.spawn(test_stream_out(&client)),
        executor.spawn(test_stream_in(&client)),
    ];
    let tests = tests
        .into_iter()
        .map(|test| timeout(Duration::from_secs(10), test));
    executor
        .run(async {
            let mut pass = true;
            for (i, test) in tests.into_iter().enumerate() {
                if test
                    .instrument(info_span!("client", test = i))
                    .await
                    .is_ok()
                {
                    info!("test {i} complete");
                } else {
                    pass = false;
                    info!("test {i} timed out");
                }
            }
            assert!(pass, "Tests did not complete successfully");
        })
        .await;
    info!("Sending shutdown signal to server");
    stop_server.send(()).unwrap();
    server.await.unwrap();
}

async fn test_simple_get(client: &impl ServiceClient) {
    let response = client.simple_get().await;
    let response = response.expect("Failed to get response");
    assert_eq!(&response, "hello world");
}

async fn test_simple_set(client: &impl ServiceClient, state: &Arc<RwLock<ServerState>>) {
    client.simple_set(42).await.expect("Failed to set value");
    assert_eq!(state.read().await.set_value, 42);
}

async fn test_stream_out(client: &impl ServiceClient) {
    let stream = client.stream_out().await.expect("Failed to set value");
    let mut stream = pin!(stream);
    for i in 0..100 {
        let value = stream.next().await.expect("stream closed early");
        assert_eq!(i, value);
    }
}

async fn test_stream_in(client: &impl ServiceClient) {
    let value = client
        .stream_in(iter(0..100))
        .await
        .expect("Failed to set value");
    assert_eq!(value, 100);
}
