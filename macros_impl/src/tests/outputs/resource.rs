#[allow(
    unused_imports,
    reason = "These might not always be used, but they should be available in this module anyway"
)]
pub use resources::{Resources, ResourcesClient, ResourcesServer};

#[allow(
    unused_imports,
    reason = "These might not always be used, but it's easier to include always"
)]
mod resources {
    use ::trait_rpc::__macro_imports::*;
    use super::*;
    /// This is the [Rpc](::trait_rpc::Rpc) definition for this service
    pub struct Resources<T: Debug>(PhantomData<fn() -> (T,)>);
    impl<T: Debug> Rpc for Resources<T>
    where
        T: Send + 'static,
    {
        type Client<_Conn: Connection, _Format> = ResourcesClient<_Conn, _Format, T>;
        type Request = Request<T>;
        type Response = Response<T>;
        fn service_name() -> &'static str {
            stringify!(Resources)
        }
        fn client<_Conn: Connection, _Format>(
            client: Client<_Conn, _Format>,
        ) -> Self::Client<_Conn, _Format> {
            ResourcesClient(client, PhantomData::<fn() -> (T,)>)
        }
    }

    impl<Server: ResourcesServer<T>, T: Debug> RpcWithServer<Server> for Resources<T>
    where
        T: Send + 'static,
    {
        type Handler<_Format> = ResourcesHandler<Server, T>;
        fn handler<_Format>(server: Server) -> Self::Handler<_Format> {
            ResourcesHandler(server, PhantomData::<fn() -> (T,)>)
        }
    }
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(crate = "::trait_rpc::serde")]
    #[serde(tag = "method", content = "args")]
    pub enum Request<T> {
        #[serde(rename = "subscribe")]
        Subscribe(),
        #[serde(rename = "list")]
        List(),
        #[serde(rename = "get")]
        Get(u64),
        #[serde(rename = "new")]
        New(T),
    }
    impl<T> ::trait_rpc::Request for Request<T> {
        fn name(&self) -> Cow<'static, str> {
            match self {
                Self::Subscribe(..) => Cow::Borrowed("subscribe"),
                Self::List(..) => Cow::Borrowed("list"),
                Self::Get(..) => Cow::Borrowed("get"),
                Self::New(..) => Cow::Borrowed("new"),
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(crate = "::trait_rpc::serde")]
    #[serde(tag = "method", content = "result")]
    pub enum Response<T> {
        #[serde(rename = "subscribe")]
        Subscribe(StreamID),
        #[serde(rename = "list")]
        List(Vec<T>),
        #[serde(rename = "get")]
        Get(Option<T>),
        #[serde(rename = "new")]
        New(()),
    }
    impl<T> Response<T> {
        fn fn_name(&self) -> &'static str {
            match self {
                Self::Subscribe(..) => "subscribe",
                Self::List(..) => "list",
                Self::Get(..) => "get",
                Self::New(..) => "new",
            }
        }
    }
    /// This is the trait which is used by the server side in order to serve the client
    pub trait ResourcesServer<T>: Send + Sync {
        fn subscribe(
            &self,
        ) -> impl Future<Output = impl futures::stream::Stream<Item = T> + MaybeSend + MaybeSync> + Send;
        fn list(&self) -> impl Future<Output = Vec<T>> + Send;
        fn get(&self, id: u64) -> impl Future<Output = Option<T>> + Send;
        fn new(&self, value: T) -> impl Future<Output = ()> + Send;
    }
    /// A [Handler](Handler) which handles requests/responses for a given service
    #[derive(Debug, Clone)]
    pub struct ResourcesHandler<_Server, T>(_Server, (PhantomData<fn() -> (T,)>));
    impl<_Server: ResourcesServer<T>, _Format, T: Debug> Handler<_Format>
        for ResourcesHandler<_Server, T>
    where
        T: Send + 'static,
        _Format: WriteFormat<T>,
        T: MaybeSync,
    {
        type Rpc = Resources<T>;
        type Server = _Server;

        fn server(&self) -> &Self::Server {
            &self.0
        }

        async fn handle<'a>(
            &'a self,
            request: Request<T>,
            controller: &impl ConnectionController<'a, _Format>,
        ) -> Response<T> {
            match request {
                Request::Subscribe() => {
                    let value = self.0.subscribe().await;
                    let value = controller.sending_stream::<T>(value).await;
                    Response::Subscribe(value)
                }
                Request::List() => {
                    let value = self.0.list().await;
                    Response::List(value)
                }
                Request::Get(id) => {
                    let value = self.0.get(id).await;
                    Response::Get(value)
                }
                Request::New(value) => {
                    let value = self.0.new(value).await;
                    Response::New(value)
                }
            }
        }
    }

    /// This is the async client for the service, it produces requests from method calls
    /// (including chained method calls) and sends the requests with the given
    /// [transport](::trait_rpc::Client) before returning the response
    ///
    /// The return value is always wrapped in a result: `Result<T, ClientError<_Conn::SendError>>` where `T` is the service return value
    #[derive(Clone)]
    pub struct ResourcesClient<_Conn, _Format, T: Debug>(
        Client<_Conn, _Format>,
        (PhantomData<fn() -> (T,)>),
    );
    #[allow(clippy::future_not_send)]
    impl<_Conn, _Format, T: Debug> ResourcesClient<_Conn, _Format, T>
    where
        _Conn: Connection + MaybeSend + 'static,
        _Format: Format<Response<T>, Request<T>> + ReadFormat<T>,
        T: MaybeSend + MaybeSync,
    {
        pub async fn subscribe(
            &self,
        ) -> Result<impl futures::stream::Stream<Item = T>, ClientError<_Conn::SendError>> {
            match self.0.send(Request::Subscribe()).await? {
                Response::Subscribe(value) => {
                    let value = self.0.receiving_stream::<T>(value).await;
                    Ok(value)
                }
                other => Err(WrongResponseType::new("subscribe", other.fn_name()).into()),
            }
        }
        pub async fn list(&self) -> Result<Vec<T>, ClientError<_Conn::SendError>> {
            match self.0.send(Request::List()).await? {
                Response::List(value) => Ok(value),
                other => Err(WrongResponseType::new("list", other.fn_name()).into()),
            }
        }
        pub async fn get(&self, id: u64) -> Result<Option<T>, ClientError<_Conn::SendError>> {
            match self.0.send(Request::Get(id)).await? {
                Response::Get(value) => Ok(value),
                other => Err(WrongResponseType::new("get", other.fn_name()).into()),
            }
        }
        pub async fn new(&self, value: T) -> Result<(), ClientError<_Conn::SendError>> {
            match self.0.send(Request::New(value)).await? {
                Response::New(value) => Ok(value),
                other => Err(WrongResponseType::new("new", other.fn_name()).into()),
            }
        }
    }
}
