#[allow(
    unused_imports,
    reason = "These might not always be used, but they should be available in this module anyway"
)]
pub use resources_service::{
    ResourcesService, ResourcesServiceClient, ResourcesServiceClientImpl, ResourcesServiceHandler,
    ResourcesServiceServer,
};
#[allow(clippy::unused_unit)]
#[allow(clippy::multiple_bound_locations)]
#[allow(
    unused_imports,
    reason = "These might not always be used, but it's easier to include always"
)]
mod resources_service {
    use super::*;
    use ::nasc::__macros::{imports, traits::*};
    #[doc = r""]
    #[doc = r" This is the [`Service`](imports::Service) definition for this service"]
    pub struct ResourcesService(imports::PhantomData<fn() -> ()>);
    impl ResourcesService {
        pub const METHOD_ID_BOOKS: imports::MethodID = imports::MethodID::_new(131445777_u32);
        pub const METHOD_ID_AUTHORS: imports::MethodID = imports::MethodID::_new(1127417919_u32);
    }
    impl imports::Service for ResourcesService {
        type Client<_Format: imports::Format> = ResourcesServiceClientImpl<_Format>;
        fn client<_Format: imports::Format>(
            client: imports::Client<Self>,
            format: _Format,
        ) -> Self::Client<_Format> {
            ResourcesServiceClientImpl {
                client,
                format,
                _generics: imports::PhantomData,
            }
        }
        fn service_name() -> &'static str {
            stringify!(ResourcesService)
        }
    }
    impl<_Server, _Format> imports::ServiceWithServer<_Server, _Format> for ResourcesService
    where
        _Format: imports::Format,
        ResourcesServiceHandler<_Server, _Format>: imports::Handler,
        _Server: ResourcesServiceServer<_Format>,
    {
        type Handler = ResourcesServiceHandler<_Server, _Format>;
        fn handler(server: _Server, format: _Format) -> Self::Handler {
            ResourcesServiceHandler {
                server,
                format,
                _generics: imports::PhantomData,
            }
        }
    }
    #[doc = r""]
    #[doc = r" This is the trait which is used by the server side in order to serve the client's"]
    #[doc = r" requests"]
    pub trait ResourcesServiceServer<_Format>: Send + Sync + 'static
    where
        _Format: imports::Format,
    {
        fn books(
            &self,
        ) -> impl imports::Future<
            Output = impl imports::IntoHandler<ResourceService<Book>, _Format>,
        > + Send;
        fn authors(
            &self,
        ) -> impl imports::Future<
            Output = impl imports::IntoHandler<ResourceService<Author>, _Format>,
        > + Send;
    }
    #[doc = " A [Handler](imports::Handler) which handles requests/responses for a given service"]
    #[derive(Debug, Clone)]
    pub struct ResourcesServiceHandler<_Server, _Format> {
        server: _Server,
        format: _Format,
        _generics: imports::PhantomData<fn() -> ()>,
    }
    impl<_Server, _Format> imports::Handler for ResourcesServiceHandler<_Server, _Format>
    where
        _Server: ResourcesServiceServer<_Format>,
        _Format: imports::ReadFormat<()>,
        _Format: imports::ReadFormat<()>,
        _Format: imports::WriteFormat<u32>,
        _Format: imports::WriteFormat<u32>,
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
                131445777_u32 => {
                    let (): () = self.format.read(&request).map_err(|error| {
                        imports::HandleError::BadPayloadFormat(error.to_string())
                    })?;
                    let value = self.server.books().await;
                    let value: u32 = manager
                        .outgoing_service(value.into_handler(format.clone()), guard.clone())
                        .await
                        .as_u32();
                    self.format.write(value).map_err(|error| {
                        imports::HandleError::InternalError(format!(
                            "Error writing response: {error}"
                        ))
                    })
                }
                1127417919_u32 => {
                    let (): () = self.format.read(&request).map_err(|error| {
                        imports::HandleError::BadPayloadFormat(error.to_string())
                    })?;
                    let value = self.server.authors().await;
                    let value: u32 = manager
                        .outgoing_service(value.into_handler(format.clone()), guard.clone())
                        .await
                        .as_u32();
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
    pub trait ResourcesServiceClient: Send + Sync {
        type Format: imports::Format;
        #[doc = r""]
        #[doc = r" Returns a future builder that allows certain options to be set before `await`-ing"]
        #[doc = r""]
        #[doc = r" see [`ClientResponseFuture`](imports::ClientResponseFuture)"]
        fn books(
            &self,
        ) -> imports::ClientResponseFuture<
            impl imports::Future<
                Output = Result<
                    <ResourceService<Book> as imports::Service>::Client<Self::Format>,
                    imports::ClientError,
                >,
            > + Send,
        >;
        #[doc = r""]
        #[doc = r" Returns a future builder that allows certain options to be set before `await`-ing"]
        #[doc = r""]
        #[doc = r" see [`ClientResponseFuture`](imports::ClientResponseFuture)"]
        fn authors(
            &self,
        ) -> imports::ClientResponseFuture<
            impl imports::Future<
                Output = Result<
                    <ResourceService<Author> as imports::Service>::Client<Self::Format>,
                    imports::ClientError,
                >,
            > + Send,
        >;
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct ResourcesServiceClientImpl<_Format> {
        client: imports::Client<ResourcesService>,
        format: _Format,
        _generics: imports::PhantomData<fn() -> ()>,
    }
    #[doc = r""]
    #[doc = r" This is the async client for the service, it produces requests from method calls"]
    #[doc = r" and sends the requests with the given [client](imports::Client) before returning"]
    #[doc = r" the response"]
    #[doc = r""]
    #[doc = r" The return value is always wrapped in a result: `Result<T, ClientError<_Conn::SendError>>`"]
    #[doc = r" where `T` is the service return value"]
    impl<_Format> ResourcesServiceClient for ResourcesServiceClientImpl<_Format>
    where
        _Format: imports::ReadFormat<u32>,
        _Format: imports::ReadFormat<u32>,
        _Format: imports::WriteFormat<()>,
        _Format: imports::WriteFormat<()>,
    {
        type Format = _Format;
        fn books(
            &self,
        ) -> imports::ClientResponseFuture<
            impl Future<
                Output = Result<
                    <ResourceService<Book> as imports::Service>::Client<Self::Format>,
                    imports::ClientError,
                >,
            > + Send,
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
                    let value: u32 = format
                        .read(&response)
                        .map_err(imports::ClientError::Deserialise)?;
                    Ok(manager
                        .incoming_service::<ResourceService<Book>, _>(
                            imports::ServiceID::from(value),
                            guard.clone(),
                            format.clone(),
                        )
                        .await)
                }
            };
            self.client
                .send(request, imports::MethodID::_new(131445777_u32), response)
        }
        fn authors(
            &self,
        ) -> imports::ClientResponseFuture<
            impl Future<
                Output = Result<
                    <ResourceService<Author> as imports::Service>::Client<Self::Format>,
                    imports::ClientError,
                >,
            > + Send,
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
                    let value: u32 = format
                        .read(&response)
                        .map_err(imports::ClientError::Deserialise)?;
                    Ok(manager
                        .incoming_service::<ResourceService<Author>, _>(
                            imports::ServiceID::from(value),
                            guard.clone(),
                            format.clone(),
                        )
                        .await)
                }
            };
            self.client
                .send(request, imports::MethodID::_new(1127417919_u32), response)
        }
    }
}
#[allow(
    unused_imports,
    reason = "These might not always be used, but they should be available in this module anyway"
)]
pub use resource_service::{
    ResourceService, ResourceServiceClient, ResourceServiceClientImpl, ResourceServiceHandler,
    ResourceServiceServer,
};
#[allow(clippy::unused_unit)]
#[allow(clippy::multiple_bound_locations)]
#[allow(
    unused_imports,
    reason = "These might not always be used, but it's easier to include always"
)]
mod resource_service {
    use super::*;
    use ::nasc::__macros::{imports, traits::*};
    #[doc = r""]
    #[doc = r" This is the [`Service`](imports::Service) definition for this service"]
    pub struct ResourceService<T>(imports::PhantomData<fn() -> (T,)>)
    where
        T: Resource;
    impl<T> ResourceService<T>
    where
        T: Resource,
    {
        pub const METHOD_ID_SUBSCRIBE: imports::MethodID = imports::MethodID::_new(2946386435_u32);
        pub const METHOD_ID_UPDATES: imports::MethodID = imports::MethodID::_new(394659589_u32);
        pub const METHOD_ID_LIST: imports::MethodID = imports::MethodID::_new(217798785_u32);
        pub const METHOD_ID_GET: imports::MethodID = imports::MethodID::_new(1410115415_u32);
        pub const METHOD_ID_NEW: imports::MethodID = imports::MethodID::_new(681154065_u32);
    }
    impl<T> imports::Service for ResourceService<T>
    where
        T: Resource,
    {
        type Client<_Format: imports::Format> = ResourceServiceClientImpl<_Format, T>;
        fn client<_Format: imports::Format>(
            client: imports::Client<Self>,
            format: _Format,
        ) -> Self::Client<_Format> {
            ResourceServiceClientImpl {
                client,
                format,
                _generics: imports::PhantomData,
            }
        }
        fn service_name() -> &'static str {
            stringify!(ResourceService)
        }
    }
    impl<_Server, _Format, T> imports::ServiceWithServer<_Server, _Format> for ResourceService<T>
    where
        _Format: imports::Format,
        ResourceServiceHandler<_Server, _Format, T>: imports::Handler,
        _Server: ResourceServiceServer<_Format, T>,
        T: Resource,
    {
        type Handler = ResourceServiceHandler<_Server, _Format, T>;
        fn handler(server: _Server, format: _Format) -> Self::Handler {
            ResourceServiceHandler {
                server,
                format,
                _generics: imports::PhantomData,
            }
        }
    }
    #[doc = r""]
    #[doc = r" This is the trait which is used by the server side in order to serve the client's"]
    #[doc = r" requests"]
    pub trait ResourceServiceServer<_Format, T>: Send + Sync + 'static
    where
        _Format: imports::Format,
        T: Resource,
    {
        fn subscribe(
            &self,
            listener: <Subscriber<T> as imports::Service>::Client<_Format>,
        ) -> impl imports::Future<Output = ()> + Send;
        fn updates(
            &self,
        ) -> impl imports::Future<Output = impl imports::Stream<Item = T> + Send + 'static> + Send;
        fn list(&self) -> impl imports::Future<Output = Vec<T>> + Send;
        fn get(&self, id: u64) -> impl imports::Future<Output = Option<T>> + Send;
        fn new(&self, value: T) -> impl imports::Future<Output = ()> + Send;
    }
    #[doc = " A [Handler](imports::Handler) which handles requests/responses for a given service"]
    #[derive(Debug, Clone)]
    pub struct ResourceServiceHandler<_Server, _Format, T>
    where
        T: Resource,
    {
        server: _Server,
        format: _Format,
        _generics: imports::PhantomData<fn() -> (T,)>,
    }
    impl<_Server, _Format, T> imports::Handler for ResourceServiceHandler<_Server, _Format, T>
    where
        _Server: ResourceServiceServer<_Format, T>,
        _Format: imports::ReadFormat<(u32,)>,
        _Format: imports::ReadFormat<()>,
        _Format: imports::ReadFormat<()>,
        _Format: imports::ReadFormat<(u64,)>,
        _Format: imports::ReadFormat<(T,)>,
        _Format: imports::WriteFormat<()>,
        _Format: imports::WriteFormat<u32>,
        _Format: imports::WriteFormat<Vec<T>>,
        _Format: imports::WriteFormat<Option<T>>,
        _Format: imports::WriteFormat<()>,
        _Format: imports::WriteFormat<T>,
        T: Resource,
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
                2946386435_u32 => {
                    let (listener,): (u32,) = self.format.read(&request).map_err(|error| {
                        imports::HandleError::BadPayloadFormat(error.to_string())
                    })?;
                    let listener = {
                        let value = listener;
                        manager
                            .incoming_service::<Subscriber<T>, _>(
                                imports::ServiceID::from(value),
                                guard.clone(),
                                format.clone(),
                            )
                            .await
                    };
                    let value = self.server.subscribe(listener).await;
                    let value: () = value;
                    self.format.write(value).map_err(|error| {
                        imports::HandleError::InternalError(format!(
                            "Error writing response: {error}"
                        ))
                    })
                }
                394659589_u32 => {
                    let (): () = self.format.read(&request).map_err(|error| {
                        imports::HandleError::BadPayloadFormat(error.to_string())
                    })?;
                    let value = self.server.updates().await;
                    let value: u32 = {
                        let format = format.clone();
                        let stream = imports::StreamExt::filter_map(value, move |item| {
                            let format = format.clone();
                            async move {
                                match format.write(item) {
                                    Ok(value) => Some(value),
                                    Err(error) => {
                                        imports::tracing::error!(
                                            "Failed to deserialise bytes from stream: {error}"
                                        );
                                        None
                                    }
                                }
                            }
                        });
                        let value: u32 =
                            manager.outgoing_stream(stream, guard.clone()).await.into();
                        value
                    };
                    self.format.write(value).map_err(|error| {
                        imports::HandleError::InternalError(format!(
                            "Error writing response: {error}"
                        ))
                    })
                }
                217798785_u32 => {
                    let (): () = self.format.read(&request).map_err(|error| {
                        imports::HandleError::BadPayloadFormat(error.to_string())
                    })?;
                    let value = self.server.list().await;
                    let value: Vec<T> = value;
                    self.format.write(value).map_err(|error| {
                        imports::HandleError::InternalError(format!(
                            "Error writing response: {error}"
                        ))
                    })
                }
                1410115415_u32 => {
                    let (id,): (u64,) = self.format.read(&request).map_err(|error| {
                        imports::HandleError::BadPayloadFormat(error.to_string())
                    })?;
                    let id = {
                        let value = id;
                        value
                    };
                    let value = self.server.get(id).await;
                    let value: Option<T> = value;
                    self.format.write(value).map_err(|error| {
                        imports::HandleError::InternalError(format!(
                            "Error writing response: {error}"
                        ))
                    })
                }
                681154065_u32 => {
                    let (value,): (T,) = self.format.read(&request).map_err(|error| {
                        imports::HandleError::BadPayloadFormat(error.to_string())
                    })?;
                    let value = {
                        let value = value;
                        value
                    };
                    let value = self.server.new(value).await;
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
    pub trait ResourceServiceClient<T>: Send + Sync
    where
        T: Resource,
    {
        type Format: imports::Format;
        #[doc = r""]
        #[doc = r" Returns a future builder that allows certain options to be set before `await`-ing"]
        #[doc = r""]
        #[doc = r" see [`ClientResponseFuture`](imports::ClientResponseFuture)"]
        fn subscribe(
            &self,
            listener: impl imports::IntoHandler<Subscriber<T>, Self::Format>,
        ) -> imports::ClientResponseFuture<
            impl imports::Future<Output = Result<(), imports::ClientError>> + Send,
        >;
        #[doc = r""]
        #[doc = r" Returns a future builder that allows certain options to be set before `await`-ing"]
        #[doc = r""]
        #[doc = r" see [`ClientResponseFuture`](imports::ClientResponseFuture)"]
        fn updates(
            &self,
        ) -> imports::ClientResponseFuture<
            impl imports::Future<Output = Result<impl imports::Stream<Item = T> + Send + 'static, imports::ClientError>>
            + Send,
        >;
        #[doc = r""]
        #[doc = r" Returns a future builder that allows certain options to be set before `await`-ing"]
        #[doc = r""]
        #[doc = r" see [`ClientResponseFuture`](imports::ClientResponseFuture)"]
        fn list(
            &self,
        ) -> imports::ClientResponseFuture<
            impl imports::Future<Output = Result<Vec<T>, imports::ClientError>> + Send,
        >;
        #[doc = r""]
        #[doc = r" Returns a future builder that allows certain options to be set before `await`-ing"]
        #[doc = r""]
        #[doc = r" see [`ClientResponseFuture`](imports::ClientResponseFuture)"]
        fn get(
            &self,
            id: u64,
        ) -> imports::ClientResponseFuture<
            impl imports::Future<Output = Result<Option<T>, imports::ClientError>> + Send,
        >;
        #[doc = r""]
        #[doc = r" Returns a future builder that allows certain options to be set before `await`-ing"]
        #[doc = r""]
        #[doc = r" see [`ClientResponseFuture`](imports::ClientResponseFuture)"]
        fn new(
            &self,
            value: T,
        ) -> imports::ClientResponseFuture<
            impl imports::Future<Output = Result<(), imports::ClientError>> + Send,
        >;
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct ResourceServiceClientImpl<_Format, T>
    where
        T: Resource,
    {
        client: imports::Client<ResourceService<T>>,
        format: _Format,
        _generics: imports::PhantomData<fn() -> (T,)>,
    }
    #[doc = r""]
    #[doc = r" This is the async client for the service, it produces requests from method calls"]
    #[doc = r" and sends the requests with the given [client](imports::Client) before returning"]
    #[doc = r" the response"]
    #[doc = r""]
    #[doc = r" The return value is always wrapped in a result: `Result<T, ClientError<_Conn::SendError>>`"]
    #[doc = r" where `T` is the service return value"]
    impl<_Format, T> ResourceServiceClient<T> for ResourceServiceClientImpl<_Format, T>
    where
        _Format: imports::ReadFormat<()>,
        _Format: imports::ReadFormat<u32>,
        _Format: imports::ReadFormat<Vec<T>>,
        _Format: imports::ReadFormat<Option<T>>,
        _Format: imports::ReadFormat<()>,
        _Format: imports::ReadFormat<T>,
        _Format: imports::WriteFormat<(u32,)>,
        _Format: imports::WriteFormat<()>,
        _Format: imports::WriteFormat<()>,
        _Format: imports::WriteFormat<(u64,)>,
        _Format: imports::WriteFormat<(T,)>,
        T: Resource,
    {
        type Format = _Format;
        fn subscribe(
            &self,
            listener: impl imports::IntoHandler<Subscriber<T>, Self::Format>,
        ) -> imports::ClientResponseFuture<
            impl Future<Output = Result<(), imports::ClientError>> + Send,
        > {
            let request = {
                let format = self.format.clone();
                let manager = self.client.service_manager();
                let guard = self.client.guard.clone();
                async move {
                    let value = ({
                        let value = listener;
                        manager
                            .outgoing_service(value.into_handler(format.clone()), guard.clone())
                            .await
                            .as_u32()
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
                .send(request, imports::MethodID::_new(2946386435_u32), response)
        }
        fn updates(
            &self,
        ) -> imports::ClientResponseFuture<
            impl Future<Output = Result<impl imports::Stream<Item = T>, imports::ClientError>> + Send,
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
                    let value: u32 = format
                        .read(&response)
                        .map_err(imports::ClientError::Deserialise)?;
                    Ok({
                        let format = format.clone();
                        imports::StreamExt::filter_map(
                            manager
                                .incoming_stream(imports::StreamID::from(value), guard.clone())
                                .await,
                            move |bytes| {
                                let format = format.clone();
                                async move {
                                    match imports::ReadFormat::<T>::read(&format, &bytes) {
                                        Ok(value) => Some(value),
                                        Err(error) => {
                                            imports::tracing::error!(
                                                "Failed to deserialise bytes from stream: {error}"
                                            );
                                            None
                                        }
                                    }
                                }
                            },
                        )
                    })
                }
            };
            self.client
                .send(request, imports::MethodID::_new(394659589_u32), response)
        }
        fn list(
            &self,
        ) -> imports::ClientResponseFuture<
            impl Future<Output = Result<Vec<T>, imports::ClientError>> + Send,
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
                    let value: Vec<T> = format
                        .read(&response)
                        .map_err(imports::ClientError::Deserialise)?;
                    Ok(value)
                }
            };
            self.client
                .send(request, imports::MethodID::_new(217798785_u32), response)
        }
        fn get(
            &self,
            id: u64,
        ) -> imports::ClientResponseFuture<
            impl Future<Output = Result<Option<T>, imports::ClientError>> + Send,
        > {
            let request = {
                let format = self.format.clone();
                let manager = self.client.service_manager();
                let guard = self.client.guard.clone();
                async move {
                    let value = ({
                        let value = id;
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
                    let value: Option<T> = format
                        .read(&response)
                        .map_err(imports::ClientError::Deserialise)?;
                    Ok(value)
                }
            };
            self.client
                .send(request, imports::MethodID::_new(1410115415_u32), response)
        }
        fn new(
            &self,
            value: T,
        ) -> imports::ClientResponseFuture<
            impl Future<Output = Result<(), imports::ClientError>> + Send,
        > {
            let request = {
                let format = self.format.clone();
                let manager = self.client.service_manager();
                let guard = self.client.guard.clone();
                async move {
                    let value = ({
                        let value = value;
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
                .send(request, imports::MethodID::_new(681154065_u32), response)
        }
    }
}
#[allow(
    unused_imports,
    reason = "These might not always be used, but they should be available in this module anyway"
)]
pub use subscriber::{
    Subscriber, SubscriberClient, SubscriberClientImpl, SubscriberHandler, SubscriberServer,
};
#[allow(clippy::unused_unit)]
#[allow(clippy::multiple_bound_locations)]
#[allow(
    unused_imports,
    reason = "These might not always be used, but it's easier to include always"
)]
mod subscriber {
    use super::*;
    use ::nasc::__macros::{imports, traits::*};
    #[doc = r""]
    #[doc = r" This is the [`Service`](imports::Service) definition for this service"]
    pub struct Subscriber<T>(imports::PhantomData<fn() -> (T,)>)
    where
        T: Resource;
    impl<T> Subscriber<T>
    where
        T: Resource,
    {
        pub const METHOD_ID_DELETED: imports::MethodID = imports::MethodID::_new(2228311146_u32);
        pub const METHOD_ID_UPDATED: imports::MethodID = imports::MethodID::_new(545658160_u32);
        pub const METHOD_ID_NEW: imports::MethodID = imports::MethodID::_new(681154065_u32);
    }
    impl<T> imports::Service for Subscriber<T>
    where
        T: Resource,
    {
        type Client<_Format: imports::Format> = SubscriberClientImpl<_Format, T>;
        fn client<_Format: imports::Format>(
            client: imports::Client<Self>,
            format: _Format,
        ) -> Self::Client<_Format> {
            SubscriberClientImpl {
                client,
                format,
                _generics: imports::PhantomData,
            }
        }
        fn service_name() -> &'static str {
            stringify!(Subscriber)
        }
    }
    impl<_Server, _Format, T> imports::ServiceWithServer<_Server, _Format> for Subscriber<T>
    where
        _Format: imports::Format,
        SubscriberHandler<_Server, _Format, T>: imports::Handler,
        _Server: SubscriberServer<_Format, T>,
        T: Resource,
    {
        type Handler = SubscriberHandler<_Server, _Format, T>;
        fn handler(server: _Server, format: _Format) -> Self::Handler {
            SubscriberHandler {
                server,
                format,
                _generics: imports::PhantomData,
            }
        }
    }
    #[doc = r""]
    #[doc = r" This is the trait which is used by the server side in order to serve the client's"]
    #[doc = r" requests"]
    pub trait SubscriberServer<_Format, T>: Send + Sync + 'static
    where
        _Format: imports::Format,
        T: Resource,
    {
        fn deleted(&self, value: T) -> impl imports::Future<Output = ()> + Send;
        fn updated(&self, value: T) -> impl imports::Future<Output = ()> + Send;
        fn new(&self, value: T) -> impl imports::Future<Output = ()> + Send;
    }
    #[doc = " A [Handler](imports::Handler) which handles requests/responses for a given service"]
    #[derive(Debug, Clone)]
    pub struct SubscriberHandler<_Server, _Format, T>
    where
        T: Resource,
    {
        server: _Server,
        format: _Format,
        _generics: imports::PhantomData<fn() -> (T,)>,
    }
    impl<_Server, _Format, T> imports::Handler for SubscriberHandler<_Server, _Format, T>
    where
        _Server: SubscriberServer<_Format, T>,
        _Format: imports::ReadFormat<(T,)>,
        _Format: imports::ReadFormat<(T,)>,
        _Format: imports::ReadFormat<(T,)>,
        _Format: imports::WriteFormat<()>,
        _Format: imports::WriteFormat<()>,
        _Format: imports::WriteFormat<()>,
        T: Resource,
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
                2228311146_u32 => {
                    let (value,): (T,) = self.format.read(&request).map_err(|error| {
                        imports::HandleError::BadPayloadFormat(error.to_string())
                    })?;
                    let value = {
                        let value = value;
                        value
                    };
                    let value = self.server.deleted(value).await;
                    let value: () = value;
                    self.format.write(value).map_err(|error| {
                        imports::HandleError::InternalError(format!(
                            "Error writing response: {error}"
                        ))
                    })
                }
                545658160_u32 => {
                    let (value,): (T,) = self.format.read(&request).map_err(|error| {
                        imports::HandleError::BadPayloadFormat(error.to_string())
                    })?;
                    let value = {
                        let value = value;
                        value
                    };
                    let value = self.server.updated(value).await;
                    let value: () = value;
                    self.format.write(value).map_err(|error| {
                        imports::HandleError::InternalError(format!(
                            "Error writing response: {error}"
                        ))
                    })
                }
                681154065_u32 => {
                    let (value,): (T,) = self.format.read(&request).map_err(|error| {
                        imports::HandleError::BadPayloadFormat(error.to_string())
                    })?;
                    let value = {
                        let value = value;
                        value
                    };
                    let value = self.server.new(value).await;
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
    pub trait SubscriberClient<T>: Send + Sync
    where
        T: Resource,
    {
        type Format: imports::Format;
        #[doc = r""]
        #[doc = r" Returns a future builder that allows certain options to be set before `await`-ing"]
        #[doc = r""]
        #[doc = r" see [`ClientResponseFuture`](imports::ClientResponseFuture)"]
        fn deleted(
            &self,
            value: T,
        ) -> imports::ClientResponseFuture<
            impl imports::Future<Output = Result<(), imports::ClientError>> + Send,
        >;
        #[doc = r""]
        #[doc = r" Returns a future builder that allows certain options to be set before `await`-ing"]
        #[doc = r""]
        #[doc = r" see [`ClientResponseFuture`](imports::ClientResponseFuture)"]
        fn updated(
            &self,
            value: T,
        ) -> imports::ClientResponseFuture<
            impl imports::Future<Output = Result<(), imports::ClientError>> + Send,
        >;
        #[doc = r""]
        #[doc = r" Returns a future builder that allows certain options to be set before `await`-ing"]
        #[doc = r""]
        #[doc = r" see [`ClientResponseFuture`](imports::ClientResponseFuture)"]
        fn new(
            &self,
            value: T,
        ) -> imports::ClientResponseFuture<
            impl imports::Future<Output = Result<(), imports::ClientError>> + Send,
        >;
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct SubscriberClientImpl<_Format, T>
    where
        T: Resource,
    {
        client: imports::Client<Subscriber<T>>,
        format: _Format,
        _generics: imports::PhantomData<fn() -> (T,)>,
    }
    #[doc = r""]
    #[doc = r" This is the async client for the service, it produces requests from method calls"]
    #[doc = r" and sends the requests with the given [client](imports::Client) before returning"]
    #[doc = r" the response"]
    #[doc = r""]
    #[doc = r" The return value is always wrapped in a result: `Result<T, ClientError<_Conn::SendError>>`"]
    #[doc = r" where `T` is the service return value"]
    impl<_Format, T> SubscriberClient<T> for SubscriberClientImpl<_Format, T>
    where
        _Format: imports::ReadFormat<()>,
        _Format: imports::ReadFormat<()>,
        _Format: imports::ReadFormat<()>,
        _Format: imports::WriteFormat<(T,)>,
        _Format: imports::WriteFormat<(T,)>,
        _Format: imports::WriteFormat<(T,)>,
        T: Resource,
    {
        type Format = _Format;
        fn deleted(
            &self,
            value: T,
        ) -> imports::ClientResponseFuture<
            impl Future<Output = Result<(), imports::ClientError>> + Send,
        > {
            let request = {
                let format = self.format.clone();
                let manager = self.client.service_manager();
                let guard = self.client.guard.clone();
                async move {
                    let value = ({
                        let value = value;
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
                .send(request, imports::MethodID::_new(2228311146_u32), response)
        }
        fn updated(
            &self,
            value: T,
        ) -> imports::ClientResponseFuture<
            impl Future<Output = Result<(), imports::ClientError>> + Send,
        > {
            let request = {
                let format = self.format.clone();
                let manager = self.client.service_manager();
                let guard = self.client.guard.clone();
                async move {
                    let value = ({
                        let value = value;
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
                .send(request, imports::MethodID::_new(545658160_u32), response)
        }
        fn new(
            &self,
            value: T,
        ) -> imports::ClientResponseFuture<
            impl Future<Output = Result<(), imports::ClientError>> + Send,
        > {
            let request = {
                let format = self.format.clone();
                let manager = self.client.service_manager();
                let guard = self.client.guard.clone();
                async move {
                    let value = ({
                        let value = value;
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
                .send(request, imports::MethodID::_new(681154065_u32), response)
        }
    }
}
