use axum::extract::WebSocketUpgrade;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use derive_more::{AsRef, Deref};
use futures::StreamExt;
use log::Level;
use nasc::__macros::imports::Stream;
use nasc::connection::ConnectionOptions;
use nasc::format::cbor::Cbor;
use nasc::format::json::Json;
use nasc::format::FormatSet;
use nasc::websocket::axum::{handle_websocket, WebsocketError};
use nasc::websocket::tokio_tungstenite::new_websocket_transport;
use nasc::{service, IntoHandler};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::time::sleep;
use tokio_stream::wrappers::BroadcastStream;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct Book {
    pub id: u64,
    pub title: String,
    pub author: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct Author {
    pub id: u64,
    pub name: String,
    pub quote: String,
}

#[service]
pub trait ResourcesService {
    fn books() -> impl Service<ResourceService<Book>>;
    fn authors() -> impl Service<ResourceService<Author>>;
}

#[service]
pub trait ResourceService<T: Resource> {
    fn subscribe(listener: impl Service<Subscriber<T>>);
    fn updates() ->impl Stream<Item=T>;
    fn list() -> Vec<T>;
    fn get(id: u64) -> Option<T>;
    fn new(value: T);
}

#[service]
pub trait Subscriber<T: Resource> {
    fn deleted(value: T);
    fn updated(value: T);
    fn new(value: T);
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

#[tokio::test]
async fn resources() {
    simple_logger::init_with_level(Level::Info).unwrap();

    let state = State::default();
    let app = Router::new()
        .route("/api", get(handle))
        .with_state(state);

    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
            .await
            .unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    sleep(Duration::from_secs(1)).await;

    let client = {
        let options = ConnectionOptions::builder()
            .default_client_timeout(Duration::from_secs(10)) // <-- set default request timeout
            .build();
        new_websocket_transport::<ResourcesService,_>("ws://localhost:8080/api", Json, ||{
            println!("Client closed");
        }, options).await.unwrap()
    };
    
    let authors = client.authors().await.unwrap();
    authors.new(Author {
        id: 0,
        name: "John B. Keane".to_string(),
        quote: "Land is all that matters, Tadhg boy, own your own land.".to_string(),
    }).await.unwrap();

    {
        // drop a clone: This should not have any effect on the original
        let _cloned = authors.clone();
    }

    let john = authors.get(0).await.unwrap().expect("Author not found");
    assert_eq!(john.id, 0);
    assert_eq!(john.name, "John B. Keane");

    
}
