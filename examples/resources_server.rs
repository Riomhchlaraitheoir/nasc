#![doc = include_str!("./examples.md")]

use axum::Router;
use axum::extract::{FromRequestParts, WebSocketUpgrade};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use derive_more::{AsRef, Deref};
use futures::{Stream};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tokio_stream::StreamExt;
use tracing::warn;
use trait_rpc::{MaybeSend, MaybeSync};
use trait_rpc::format::cbor::Cbor;
use trait_rpc::format::json::Json;
use trait_rpc::server::ServerHooks;
use trait_rpc::server::axum::{WebsocketError, handle_websocket};

include!("traits/resources.rs");

#[tokio::main]
async fn main() {
    let state = State::default();
    let app = Router::new()
        .route("/api/books", post(handle::<Book>))
        .route("/api/authors", post(handle::<Author>))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle<R: Debug + Resource + Serialize + DeserializeOwned>(conn: WebSocketUpgrade, server: ResourceServer<R>) -> impl IntoResponse {
    let result =
        handle_websocket::<Resources<R>, _, _>(conn, server, &(Json, Cbor));
    match result {
        Ok(response) => response,
        Err(WebsocketError::UnsupportedFormat) => StatusCode::BAD_REQUEST.into_response(),
    }
}

#[derive(AsRef, Default, Clone)]
struct State {
    #[as_ref]
    books: ResourceState<Book>,
    #[as_ref]
    authors: ResourceState<Author>,
}

struct ResourceState<T> {
    map: Arc<RwLock<HashMap<u64, T>>>,
    new: broadcast::Sender<T>,
}

impl<T> Clone for ResourceState<T> {
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
            new: self.new.clone(),
        }
    }
}

#[derive(Deref)]
struct ResourceServer<T> {
    state: ResourceState<T>,
}

impl<T: Resource> Default for ResourceState<T> {
    fn default() -> Self {
        Self {
            map: Arc::default(),
            new: broadcast::channel(10).0,
        }
    }
}

impl<T> FromRequestParts<State> for ResourceServer<T>
where
    State: AsRef<ResourceState<T>>,
{
    type Rejection = Infallible;

    async fn from_request_parts(_: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let state = state.as_ref();
        Ok(Self {
            state: state.clone(),
        })
    }
}

trait Resource: Clone + Send + Sync + 'static {
    fn id(&self) -> u64;
}

impl Resource for Book {
    fn id(&self) -> u64 {
        self.id
    }
}

impl Resource for Author {
    fn id(&self) -> u64 {
        self.id
    }
}

impl<T: Resource> ResourcesServer<T> for ResourceServer<T> {
    async fn subscribe(&self) -> impl Stream<Item=T> + MaybeSend {
        tokio_stream::wrappers::BroadcastStream::new(self.new.subscribe()).filter_map(|result| {
            match result {
                Ok(res) => Some(res),
                Err(error) => {
                    warn!(?error, "client lagging on subscribe reading");
                    None
                }
            }
        })
    }

    async fn list(&self) -> Vec<T> {
        self.map.read().await.values().cloned().collect()
    }

    async fn get(&self, id: u64) -> Option<T> {
        self.map.read().await.get(&id).cloned()
    }

    async fn new(&self, value: T) {
        self.map.write().await.insert(value.id(), value);
    }
}

impl<R: Resource + Debug> ServerHooks<Resources<R>> for ResourceServer<R> {}
