#[allow(
    unused_imports,
    reason = "These might not always be used, but they should be available in this module anyway"
)]
pub use todo_service::{
    TodoService, TodoServiceClient, TodoServiceServer,
};

#[allow(
    unused_imports,
    reason = "These might not always be used, but it's easier to include always"
)]
mod todo_service {
    use ::trait_rpc::__macro_imports::*;
    use super::*;

    /// A service for managing to-do items
    ///
    /// This is the [Rpc](::trait_rpc::Rpc) definition for this service
    pub struct TodoService;

    impl Rpc for TodoService {
        type Client<_Conn: Connection, _Format> =
            TodoServiceClient<_Conn, _Format>;
        type Request = Request;
        type Response = Response;
        fn service_name() -> &'static str {
            stringify!(TodoService)
        }
        fn client<_Conn: Connection, _Format>(client: Client<_Conn, _Format>) -> Self::Client<_Conn, _Format> {
            TodoServiceClient(client)
        }
    }

    impl<Server: TodoServiceServer> RpcWithServer<Server> for TodoService {
        type Handler<_Format> = TodoServiceHandler<Server>;
        fn handler<_Format>(server: Server) -> Self::Handler<_Format> {
            TodoServiceHandler(server)
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(crate = "::trait_rpc::serde")]
    #[serde(tag = "method", content = "args")]
    pub enum Request {
        #[serde(rename = "get_todos")]
        GetTodos(),
        #[serde(rename = "get_todo")]
        GetTodo(String),
        #[serde(rename = "new_todo")]
        NewTodo(Todo),
    }
    impl ::trait_rpc::Request for Request {
        fn name(&self) -> Cow<'static, str> {
            match self {
                Self::GetTodos(..) => Cow::Borrowed("get_todos"),
                Self::GetTodo(..) => Cow::Borrowed("get_todo"),
                Self::NewTodo(..) => Cow::Borrowed("new_todo"),
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(crate = "::trait_rpc::serde")]
    #[serde(tag = "method", content = "result")]
    pub enum Response {
        #[serde(rename = "get_todos")]
        GetTodos(Vec<Todo>),
        #[serde(rename = "get_todo")]
        GetTodo(Option<Todo>),
        #[serde(rename = "new_todo")]
        NewTodo(()),
    }
    impl Response {
        fn fn_name(&self) -> &'static str {
            match self {
                Self::GetTodos(..) => "get_todos",
                Self::GetTodo(..) => "get_todo",
                Self::NewTodo(..) => "new_todo",
            }
        }
    }

    /// A service for managing to-do items
    ///
    /// This is the trait which is used by the server side in order to serve the client
    pub trait TodoServiceServer: Send + Sync {
        /// Get a list of to-do items
        fn get_todos(&self) -> impl Future<Output = Vec<Todo>> + Send;
        /// Get a to-do item by name, returns None if no to-do item with the given name exists
        fn get_todo(&self, name: String) -> impl Future<Output = Option<Todo>> + Send;
        /// Create a new to-do item
        fn new_todo(&self, todo: Todo) -> impl Future<Output = ()> + Send;
    }

    /// A [Handler](Handler) which handles requests/responses for a given service
    #[derive(Debug, Clone)]
    pub struct TodoServiceHandler<_Server>(_Server);

    impl<_Server: TodoServiceServer, _Format> Handler<_Format> for TodoServiceHandler<_Server> {
        type Rpc = TodoService;
        type Server = _Server;

        fn server(&self) -> &Self::Server {
            &self.0
        }

        async fn handle<'a>(&'a self, request: Request, controller: &impl ConnectionController<'a, _Format>) -> Response {
            match request {
                Request::GetTodos() => {
                    let value = self.0.get_todos().await;
                    Response::GetTodos(value)
                },
                Request::GetTodo(name) => {
                    let value = self.0.get_todo(name).await;
                    Response::GetTodo(value)
                },
                Request::NewTodo(todo) => {
                    let value = self.0.new_todo(todo).await;
                    Response::NewTodo(value)
                },
            }
        }
    }

    /// A service for managing to-do items
    ///
    /// This is the async client for the service, it produces requests from method calls
    /// (including chained method calls) and sends the requests with the given
    /// [transport](::trait_rpc::Client) before returning the response
    ///
    /// The return value is always wrapped in a result: `Result<T, ClientError<_Conn::SendError>>` where `T` is the service return value
    #[derive(Clone)]
    pub struct TodoServiceClient<_Conn, _Format>(Client<_Conn, _Format>);

    #[allow(clippy::future_not_send)]
    impl<_Conn, _Format> TodoServiceClient<_Conn, _Format>
    where
        _Conn: Connection + MaybeSend + 'static,
        _Format: Format<Response, Request>
    {
        /// Get a list of to-do items
        pub async fn get_todos(&self) -> Result<Vec<Todo>, ClientError<_Conn::SendError>> {
            match self.0.send(Request::GetTodos()).await? {
                Response::GetTodos(value) => Ok(value),
                other => Err(WrongResponseType::new("get_todos", other.fn_name()).into()),
            }
        }
        /// Get a to-do item by name, returns None if no to-do item with the given name exists
        pub async fn get_todo(&self, name: String) -> Result<Option<Todo>, ClientError<_Conn::SendError>> {
            match self.0.send(Request::GetTodo(name)).await? {
                Response::GetTodo(value) => Ok(value),
                other => Err(WrongResponseType::new("get_todo", other.fn_name()).into()),
            }
        }
        /// Create a new to-do item
        pub async fn new_todo(&self, todo: Todo) -> Result<(), ClientError<_Conn::SendError>> {
            match self.0.send(Request::NewTodo(todo)).await? {
                Response::NewTodo(value) => Ok(value),
                other => Err(WrongResponseType::new("new_todo", other.fn_name()).into()),
            }
        }
    }
}
