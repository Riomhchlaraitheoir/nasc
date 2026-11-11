#![doc = include_str!("./examples.md")]

use axum::Router;
use axum::extract::{WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use derive_more::{AsRef, Deref};
use nasc::IntoHandler;
use nasc::connection::{ConnectionOptions, MethodID};
use nasc::format::cbor::Cbor;
use nasc::format::json::Json;
use nasc::format::{FormatSet};
use nasc::websocket::axum::{WebsocketError, handle_websocket};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tokio_stream::wrappers::BroadcastStream;
use futures::{Stream, StreamExt};
use macros::method_id;

include!("traits/resources.rs");

#[tokio::main]
async fn main() {
    let _: MethodID = method_id!(ResourceService<Book>::get);
    let state = State::default();
    let app = Router::new()
        .route("/api", post(handle))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle(conn: WebSocketUpgrade, state: axum::extract::State<State>) -> impl IntoResponse {
    let server = ResourcesServer {
        state: state.0,
    };
    let result =
        handle_websocket::<ResourcesService, _>(conn, server, &(Json, Cbor), ConnectionOptions::default());
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

/// type alias to refer to any of the formats in the format set
type AnyFormat = <(Json, Cbor) as FormatSet>::Format;

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

struct ResourcesServer {
    state: State,
}

#[derive(Deref)]
struct ResourceServer<T: Resource> {
    subscribers: RwLock<Vec<SubscriberClientImpl<AnyFormat, T>>>,
    new_resources: broadcast::Sender<T>,
    #[deref]
    state: ResourceState<T>,
}

impl ResourcesServiceServer<AnyFormat> for ResourcesServer {
    #[allow(refining_impl_trait, reason = "demonstration only; is in example")]
    async fn books(&self) -> ResourceServer<Book> {
        ResourceServer {
            subscribers: RwLock::default(),
            new_resources: broadcast::channel(10).0,
            state: self.state.books.clone(),

        }
    }
    async fn authors(&self) -> impl IntoHandler<ResourceService<Author>, AnyFormat> {
        ResourceServer {
            subscribers: RwLock::default(),
            new_resources: broadcast::channel(10).0,
            state: self.state.authors.clone(),
        }
    }
}

impl<T: Resource> Default for ResourceState<T> {
    fn default() -> Self {
        Self {
            map: Arc::default(),
            new: broadcast::channel(10).0,
        }
    }
}

trait Resource: Serialize + DeserializeOwned + Clone + Debug + Send + Sync + 'static {
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

impl<T: Resource> ResourceServiceServer<AnyFormat, T> for ResourceServer<T> {
    async fn subscribe(&self, subscriber: SubscriberClientImpl<AnyFormat, T>) {
        self.subscribers.write().await.push(subscriber);
    }

    async fn updates(&self) -> impl Stream<Item=T> + Send + 'static {
        BroadcastStream::new(self.new_resources.subscribe()).filter_map(|res| async move {
            res.ok()
        })
    }

    async fn list(&self) -> Vec<T> {
        self.map.read().await.values().cloned().collect()
    }

    async fn get(&self, id: u64) -> Option<T> {
        self.map.read().await.get(&id).cloned()
    }

    async fn new(&self, value: T) {
        self.map.write().await.insert(value.id(), value.clone());
        let _ = self.new_resources.send(value);
    }
}
