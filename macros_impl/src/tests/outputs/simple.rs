#[allow(
    unused_imports,
    reason = "These might not always be used, but they should be available in this module anyway"
)]
use todo_service::{
    TodoService, TodoServiceClient, TodoServiceClientImpl, TodoServiceHandler, TodoServiceServer,
};
#[allow(clippy::unused_unit)]
#[allow(clippy::multiple_bound_locations)]
#[allow(
    unused_imports,
    reason = "These might not always be used, but it's easier to include always"
)]
mod todo_service {
    use super::*;
    use ::nasc::__macros::{imports, traits::*};
    #[doc = " A service for managing to-do items"]
    #[doc = r""]
    #[doc = r" This is the [`Service`](imports::Service) definition for this service"]
    pub struct TodoService(imports::PhantomData<fn() -> ()>);
    impl TodoService {
        pub const METHOD_ID_GET_TODOS: imports::MethodID = imports::MethodID::_new(1521530665_u32);
        pub const METHOD_ID_GET_TODO: imports::MethodID = imports::MethodID::_new(4087466400_u32);
        pub const METHOD_ID_NEW_TODO: imports::MethodID = imports::MethodID::_new(365163866_u32);
    }
    impl imports::Service for TodoService {
        type Client<_Format: imports::Format> = TodoServiceClientImpl<_Format>;
        fn client<_Format: imports::Format>(
            client: imports::Client<Self>,
            format: _Format,
        ) -> Self::Client<_Format> {
            TodoServiceClientImpl { client, format, _generics: imports::PhantomData }
        }
        fn service_name() -> &'static str {
            stringify!(TodoService)
        }
    }
    impl<_Server, _Format> imports::ServiceWithServer<_Server, _Format>
        for TodoService
    where
        _Format: imports::Format,
        TodoServiceHandler<_Server, _Format>: imports::Handler,
        _Server: TodoServiceServer<_Format>
    {
        type Handler = TodoServiceHandler<_Server, _Format>;
        fn handler(server: _Server, format: _Format) -> Self::Handler {
            TodoServiceHandler { server, format, _generics: imports::PhantomData }
        }
    }
    #[doc = " A service for managing to-do items"]
    #[doc = r""]
    #[doc = r" This is the trait which is used by the server side in order to serve the client's"]
    #[doc = r" requests"]
    pub trait TodoServiceServer<_Format>: Send + Sync + 'static
    where
        _Format: imports::Format
    {
        #[doc = " Get a list of to-do items"]
        fn get_todos(&self) -> impl imports::Future<Output = Vec<Todo>> + Send;
        #[doc = " Get a to-do item by name, returns None if no to-do item with the given name exists"]
        fn get_todo(&self, name: String) -> impl imports::Future<Output = Option<Todo>> + Send;
        #[doc = " Create a new to-do item"]
        fn new_todo(&self, todo: Todo) -> impl imports::Future<Output = ()> + Send;
    }
    #[doc = " A [Handler](imports::Handler) which handles requests/responses for a given service"]
    #[derive(Debug, Clone)]
    pub struct TodoServiceHandler<_Server, _Format> {
        server: _Server,
        format: _Format,
        _generics: imports::PhantomData<fn() -> ()>
    }
    impl<_Server, _Format> imports::Handler for TodoServiceHandler<_Server, _Format>
    where
        _Server: TodoServiceServer<_Format>,
        _Format: imports::ReadFormat<()>,
        _Format: imports::ReadFormat<(String,)>,
        _Format: imports::ReadFormat<(Todo,)>,
        _Format: imports::WriteFormat<Vec<Todo>>,
        _Format: imports::WriteFormat<Option<Todo>>,
        _Format: imports::WriteFormat<()>,
    {
        async fn handle(
            &self,
            request_id: imports::RequestID,
            method_id: imports::MethodID,
            request: Vec<u8>,
            manager: imports::ServiceManager,
            guard: imports::TeardownGuard,
        ) -> Result<Vec<u8>, imports::HandleError> {
            let format = &self.format;
            match method_id.value() {
                1521530665_u32 => {
                    let (): () = self.format.read(&request).map_err(|error| {
                        imports::HandleError::BadPayloadFormat(error.to_string())
                    })?;
                    let value = self.server.get_todos().await;
                    let value: Vec<Todo> = value;
                    self.format.write(value).map_err(|error| {
                        imports::HandleError::InternalError(format!(
                            "Error writing response: {error}"
                        ))
                    })
                }
                4087466400_u32 => {
                    let (name,): (String,) = self.format.read(&request).map_err(|error| {
                        imports::HandleError::BadPayloadFormat(error.to_string())
                    })?;
                    let name = {
                        let value = name;
                        value
                    };
                    let value = self.server.get_todo(name).await;
                    let value: Option<Todo> = value;
                    self.format.write(value).map_err(|error| {
                        imports::HandleError::InternalError(format!(
                            "Error writing response: {error}"
                        ))
                    })
                }
                365163866_u32 => {
                    let (todo,): (Todo,) = self.format.read(&request).map_err(|error| {
                        imports::HandleError::BadPayloadFormat(error.to_string())
                    })?;
                    let todo = {
                        let value = todo;
                        value
                    };
                    let value = self.server.new_todo(todo).await;
                    let value: () = value;
                    self.format.write(value).map_err(|error| {
                        imports::HandleError::InternalError(format!(
                            "Error writing response: {error}"
                        ))
                    })
                }
                _ => return Err(imports::HandleError::MethodNotFound),
            }
        }
    }
    pub trait TodoServiceClient: Send + Sync {
        type Format: imports::Format;
        #[doc = " Get a list of to-do items"]
        #[doc = r""]
        #[doc = r" Returns a future builder that allows certain options to be set before `await`-ing"]
        #[doc = r""]
        #[doc = r" see [`ClientResponseFuture`](imports::ClientResponseFuture)"]
        fn get_todos(
            &self,
        ) -> imports::ClientResponseFuture<
            impl imports::Future<Output = Result<Vec<Todo>, imports::ClientError>> + Send,
        >;
        #[doc = " Get a to-do item by name, returns None if no to-do item with the given name exists"]
        #[doc = r""]
        #[doc = r" Returns a future builder that allows certain options to be set before `await`-ing"]
        #[doc = r""]
        #[doc = r" see [`ClientResponseFuture`](imports::ClientResponseFuture)"]
        fn get_todo(
            &self,
            name: String,
        ) -> imports::ClientResponseFuture<
            impl imports::Future<Output = Result<Option<Todo>, imports::ClientError>> + Send,
        >;
        #[doc = " Create a new to-do item"]
        #[doc = r""]
        #[doc = r" Returns a future builder that allows certain options to be set before `await`-ing"]
        #[doc = r""]
        #[doc = r" see [`ClientResponseFuture`](imports::ClientResponseFuture)"]
        fn new_todo(
            &self,
            todo: Todo,
        ) -> imports::ClientResponseFuture<
            impl imports::Future<Output = Result<(), imports::ClientError>> + Send,
        >;
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct TodoServiceClientImpl<_Format> {
        client: imports::Client<TodoService>,
        format: _Format,
        _generics: imports::PhantomData<fn() -> ()>
    }
    #[doc = " A service for managing to-do items"]
    #[doc = r""]
    #[doc = r" This is the async client for the service, it produces requests from method calls"]
    #[doc = r" and sends the requests with the given [client](imports::Client) before returning"]
    #[doc = r" the response"]
    #[doc = r""]
    #[doc = r" The return value is always wrapped in a result: `Result<T, ClientError<_Conn::SendError>>`"]
    #[doc = r" where `T` is the service return value"]
    impl<_Format> TodoServiceClient for TodoServiceClientImpl<_Format>
    where
        _Format: imports::ReadFormat<Vec<Todo>>,
        _Format: imports::ReadFormat<Option<Todo>>,
        _Format: imports::ReadFormat<()>,
        _Format: imports::WriteFormat<()>,
        _Format: imports::WriteFormat<(String,)>,
        _Format: imports::WriteFormat<(Todo,)>,
    {
        type Format = _Format;
        #[doc = " Get a list of to-do items"]
        fn get_todos(
            &self,
        ) -> imports::ClientResponseFuture<
            impl Future<Output = Result<Vec<Todo>, imports::ClientError>> + Send,
        > {
            let request = {
                let format = self.format.clone();
                let manager = self.client.service_manager();
                let guard = self.client.guard.clone();
                async move {
                    let value = ();
                    format.write(value).map_err(imports::ClientError::Serialise)
                }
            };
            let response = {
                let format = self.format.clone();
                let manager = self.client.service_manager();
                let guard = self.client.guard.clone();
                async move |response: Vec<u8>| -> Result<_, imports::ClientError> {
                    let value: Vec<Todo> = format
                        .read(&response)
                        .map_err(imports::ClientError::Deserialise)?;
                    Ok(value)
                }
            };
            self.client
                .send(request, imports::MethodID::_new(1521530665_u32), response)
        }
        #[doc = " Get a to-do item by name, returns None if no to-do item with the given name exists"]
        fn get_todo(
            &self,
            name: String,
        ) -> imports::ClientResponseFuture<
            impl Future<Output = Result<Option<Todo>, imports::ClientError>> + Send,
        > {
            let request = {
                let format = self.format.clone();
                let manager = self.client.service_manager();
                let guard = self.client.guard.clone();
                async move {
                    let value = ({
                        let value = name;
                        value
                    },);
                    format.write(value).map_err(imports::ClientError::Serialise)
                }
            };
            let response = {
                let format = self.format.clone();
                let manager = self.client.service_manager();
                let guard = self.client.guard.clone();
                async move |response: Vec<u8>| -> Result<_, imports::ClientError> {
                    let value: Option<Todo> = format
                        .read(&response)
                        .map_err(imports::ClientError::Deserialise)?;
                    Ok(value)
                }
            };
            self.client
                .send(request, imports::MethodID::_new(4087466400_u32), response)
        }
        #[doc = " Create a new to-do item"]
        fn new_todo(
            &self,
            todo: Todo,
        ) -> imports::ClientResponseFuture<
            impl Future<Output = Result<(), imports::ClientError>> + Send,
        > {
            let request = {
                let format = self.format.clone();
                let manager = self.client.service_manager();
                let guard = self.client.guard.clone();
                async move {
                    let value = ({
                        let value = todo;
                        value
                    },);
                    format.write(value).map_err(imports::ClientError::Serialise)
                }
            };
            let response = {
                let format = self.format.clone();
                let manager = self.client.service_manager();
                let guard = self.client.guard.clone();
                async move |response: Vec<u8>| -> Result<_, imports::ClientError> {
                    let value: () = format
                        .read(&response)
                        .map_err(imports::ClientError::Deserialise)?;
                    Ok(value)
                }
            };
            self.client
                .send(request, imports::MethodID::_new(365163866_u32), response)
        }
    }
}
