#[allow(
    unused_imports,
    reason = "These might not always be used, but they should be available in this module anyway"
)]
use streams_rpc::{StreamsRPC, StreamsRPCClient, StreamsRPCServer};
#[allow(
    unused_imports,
    reason = "These might not always be used, but it's easier to include always"
)]
mod streams_rpc {
    use ::trait_rpc::__macro_imports::*;
    use super::*;
    /// This is the [Rpc](::trait_rpc::Rpc) definition for this service
    pub struct StreamsRPC;
    impl Rpc for StreamsRPC {
        type Client<_Conn: Connection, _Format> = StreamsRPCClient<_Conn, _Format>;
        type Request = Request;
        type Response = Response;
        fn service_name() -> &'static str {
            stringify!(StreamsRPC)
        }
        fn client<_Conn: Connection, _Format>(
            client: Client<_Conn, _Format>,
        ) -> Self::Client<_Conn, _Format> {
            StreamsRPCClient(client)
        }
    }
    impl<Server: StreamsRPCServer> RpcWithServer<Server> for StreamsRPC {
        type Handler<_Format> = StreamsRPCHandler<Server>;
        fn handler<_Format>(server: Server) -> Self::Handler<_Format> {
            StreamsRPCHandler(server)
        }
    }
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(crate = "::trait_rpc::serde")]
    #[serde(tag = "method", content = "args")]
    pub enum Request {
        #[serde(rename = "receive")]
        Receive(),
        #[serde(rename = "send")]
        Send(StreamID),
        #[serde(rename = "multiple")]
        Multiple(Option<StreamID>, (Foo, StreamID)),
    }
    impl ::trait_rpc::Request for Request {
        fn name(&self) -> Cow<'static, str> {
            match self {
                Self::Receive(..) => Cow::Borrowed("receive"),
                Self::Send(..) => Cow::Borrowed("send"),
                Self::Multiple(..) => Cow::Borrowed("multiple"),
            }
        }
    }
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(crate = "::trait_rpc::serde")]
    #[serde(tag = "method", content = "result")]
    pub enum Response {
        #[serde(rename = "receive")]
        Receive(StreamID),
        #[serde(rename = "send")]
        Send(()),
        #[serde(rename = "multiple")]
        Multiple(Result<StreamID, Error>),
    }
    impl Response {
        fn fn_name(&self) -> &'static str {
            match self {
                Self::Receive(..) => "receive",
                Self::Send(..) => "send",
                Self::Multiple(..) => "multiple",
            }
        }
    }
    /// This is the trait which is used by the server side in order to serve the client
    pub trait StreamsRPCServer: Send + Sync {
        fn receive(
            &self,
        ) -> impl Future<
            Output = impl futures::stream::Stream<Item = Foo> + MaybeSend + MaybeSync,
        > + Send;
        fn send(
            &self,
            stream: impl futures::stream::Stream<Item = Foo>,
        ) -> impl Future<Output = ()> + Send;
        fn multiple(
            &self,
            option: Option<impl futures::stream::Stream<Item = Foo>>,
            tuple: (Foo, impl futures::stream::Stream<Item = Foo>),
        ) -> impl Future<
            Output = Result<
                impl futures::stream::Stream<Item = Bar> + MaybeSend + MaybeSync,
                Error,
            >,
        > + Send;
    }
    /// A [Handler](Handler) which handles requests/responses for a given service
    #[derive(Debug, Clone)]
    pub struct StreamsRPCHandler<_Server>(_Server);
    impl<_Server: StreamsRPCServer, _Format> Handler<_Format>
    for StreamsRPCHandler<_Server>
    where
        _Format: WriteFormat<Foo>,
        _Format: WriteFormat<Bar>,
        _Format: ReadFormat<Foo>,
        _Format: ReadFormat<Foo>,
        _Format: ReadFormat<Foo>,
        Foo: MaybeSync,
        Foo: MaybeSync,
        Foo: MaybeSync,
        Foo: MaybeSync,
        Bar: MaybeSync,
    {
        type Rpc = StreamsRPC;
        type Server = _Server;
        fn server(&self) -> &Self::Server {
            &self.0
        }
        async fn handle<'a>(
            &'a self,
            request: Request,
            controller: &impl ConnectionController<'a, _Format>,
        ) -> Response {
            match request {
                Request::Receive() => {
                    let value = self.0.receive().await;
                    let value = controller.sending_stream::<Foo>(value).await;
                    Response::Receive(value)
                }
                Request::Send(stream) => {
                    let stream = controller.receiving_stream::<Foo>(stream).await;
                    let value = self.0.send(stream).await;
                    Response::Send(value)
                }
                Request::Multiple(option, tuple) => {
                    let option = match option {
                        Some(option) => {
                            let option = controller
                                .receiving_stream::<Foo>(option)
                                .await;
                            Some(option)
                        }
                        None => None,
                    };
                    let (tuple_0, tuple_1) = tuple;
                    let tuple_1 = controller.receiving_stream::<Foo>(tuple_1).await;
                    let tuple = (tuple_0, tuple_1);
                    let value = self.0.multiple(option, tuple).await;
                    let value = match value {
                        Ok(value) => {
                            let value = controller.sending_stream::<Bar>(value).await;
                            Some(value)
                        }
                        Err(value) => Some(value),
                    };
                    Response::Multiple(value)
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
    pub struct StreamsRPCClient<_Conn, _Format>(Client<_Conn, _Format>);
    #[allow(clippy::future_not_send)]
    impl<_Conn, _Format> StreamsRPCClient<_Conn, _Format>
    where
        _Conn: Connection + MaybeSend + 'static,
        _Format: Format<Response, Request> + WriteFormat<Foo> + WriteFormat<Foo>
        + WriteFormat<Foo> + ReadFormat<Foo> + ReadFormat<Bar>,
        Foo: MaybeSend + MaybeSync,
        Foo: MaybeSend + MaybeSync,
        Foo: MaybeSend + MaybeSync,
        Foo: MaybeSend + MaybeSync,
        Bar: MaybeSend + MaybeSync,
    {
        pub async fn receive(
            &self,
        ) -> Result<
            impl futures::stream::Stream<Item = Foo>,
            ClientError<_Conn::SendError>,
        > {
            match self.0.send(Request::Receive()).await? {
                Response::Receive(value) => {
                    let value = self.0.receiving_stream::<Foo>(value).await;
                    Ok(value)
                }
                other => Err(WrongResponseType::new("receive", other.fn_name()).into()),
            }
        }
        pub async fn send(
            &self,
            stream: impl futures::stream::Stream<Item = Foo>,
        ) -> Result<(), ClientError<_Conn::SendError>> {
            let stream = self.0.sending_stream::<Foo>(stream).await;
            match self.0.send(Request::Send(stream)).await? {
                Response::Send(value) => Ok(value),
                other => Err(WrongResponseType::new("send", other.fn_name()).into()),
            }
        }
        pub async fn multiple(
            &self,
            option: Option<impl futures::stream::Stream<Item = Foo>>,
            tuple: (Foo, impl futures::stream::Stream<Item = Foo>),
        ) -> Result<
            Result<impl futures::stream::Stream<Item = Bar>, Error>,
            ClientError<_Conn::SendError>,
        > {
            let option = match option {
                Some(option) => {
                    let option = self.0.sending_stream::<Foo>(option).await;
                    Some(option)
                }
                None => None,
            };
            let (tuple_0, tuple_1) = tuple;
            let tuple_1 = self.0.sending_stream::<Foo>(tuple_1).await;
            let tuple = (tuple_0, tuple_1);
            match self.0.send(Request::Multiple(option, tuple)).await? {
                Response::Multiple(value) => {
                    let value = match value {
                        Ok(value) => {
                            let value = self.0.receiving_stream::<Bar>(value).await;
                            Some(value)
                        }
                        Err(value) => Some(value),
                    };
                    Ok(value)
                }
                other => Err(WrongResponseType::new("multiple", other.fn_name()).into()),
            }
        }
    }
}
