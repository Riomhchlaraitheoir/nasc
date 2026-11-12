#[allow(
    unused_imports,
    reason = "These might not always be used, but they should be available in this module anyway"
)]
use streams_rpc::{StreamsRPC, StreamsRPCClient, StreamsRPCClientImpl, StreamsRPCHandler, StreamsRPCServer};
#[allow(clippy::unused_unit)]
#[allow(clippy::multiple_bound_locations)]
#[allow(
    unused_imports,
    reason = "These might not always be used, but it's easier to include always"
)]
mod streams_rpc {
    use super::*;
    use ::nasc::__macros::{imports, traits::*};
    #[doc = r""]
    #[doc = r" This is the [`Service`](imports::Service) definition for this service"]
    pub struct StreamsRPC<> (imports::PhantomData<fn() -> ()>);
    impl<> StreamsRPC<> {
        pub const METHOD_ID_RECEIVE: imports::MethodID = imports::MethodID::_new(4258776482_u32);
        pub const METHOD_ID_SEND: imports::MethodID = imports::MethodID::_new(1919010991_u32);
        pub const METHOD_ID_MULTIPLE: imports::MethodID = imports::MethodID::_new(328483777_u32);
    }
    impl<> imports::Service for StreamsRPC<> {
        type Client<_Format: imports::Format> = StreamsRPCClientImpl<_Format, >;
        fn client<_Format: imports::Format>(client: imports::Client<Self>, format: _Format) -> Self::Client<_Format> { StreamsRPCClientImpl { client: ::std::sync::Arc::new(client), format, _generics: imports::PhantomData } }
        fn service_name() -> &'static str { stringify!(StreamsRPC) }
    }
    impl<_Server, _Format, > imports::ServiceWithServer<_Server, _Format> for StreamsRPC<>
    where
        _Format: imports::Format,
        StreamsRPCHandler<_Server, _Format, >: imports::Handler,
        _Server: StreamsRPCServer<_Format, >,
    {
        type Handler = StreamsRPCHandler<_Server, _Format, >;
        fn handler(server: _Server, format: _Format) -> Self::Handler { StreamsRPCHandler { server, format, _generics: imports::PhantomData } }
    }
    #[doc = r""]
    #[doc = r" This is the trait which is used by the server side in order to serve the client's"]
    #[doc = r" requests"]
    pub trait StreamsRPCServer<_Format, >: Send + Sync + 'static
    where
        _Format: imports::Format,
    {
        fn receive(&self) -> impl imports::Future<Output=impl imports::Stream<Item=Foo> + Send + 'static> + Send;
        fn send(&self, stream: impl imports::Stream<Item=Foo> + Send + 'static) -> impl imports::Future<Output=()> + Send;
        fn multiple(&self, option: Option<impl imports::Stream<Item=Foo> + Send + 'static>, tuple: (Foo, impl imports::Stream<Item=Foo> + Send + 'static,)) -> impl imports::Future<Output=Result<impl imports::Stream<Item=Bar> + Send + 'static, Error>> + Send;
    }
    #[doc = " A [Handler](imports::Handler) which handles requests/responses for a given service"]
    #[derive(Debug, Clone)]
    pub struct StreamsRPCHandler<_Server, _Format, > {
        server: _Server,
        format: _Format,
        _generics: imports::PhantomData<fn() -> ()>,
    }
    impl<_Server, _Format, > imports::Handler for StreamsRPCHandler<_Server, _Format, >
    where
        _Server: StreamsRPCServer<_Format, >,
        _Format: imports::ReadFormat<()>,
        _Format: imports::ReadFormat<(u32,)>,
        _Format: imports::ReadFormat<(Option<u32>, (Foo, u32,),)>,
        _Format: imports::ReadFormat<Foo>,
        _Format: imports::ReadFormat<Foo>,
        _Format: imports::ReadFormat<Foo>,
        _Format: imports::WriteFormat<u32>,
        _Format: imports::WriteFormat<()>,
        _Format: imports::WriteFormat<Result<u32, Error>>,
        _Format: imports::WriteFormat<Foo>,
        _Format: imports::WriteFormat<Bar>,
    {
        async fn handle(&self, request_id: imports::RequestID, method_id: imports::MethodID, request: Vec<u8>, manager: imports::ServiceManager, guard: imports::TeardownGuard) -> Result<Vec<u8>, imports::HandleError> {
            let format = &self.format;
            match method_id.value() {
                4258776482_u32 => {
                    let (): () = self.format.read(&request).map_err(|error| { imports::HandleError::BadPayloadFormat(error.to_string()) })?;
                    let value = self.server.receive().await;
                    let value: u32 = {
                        let format = format.clone();
                        let stream = imports::StreamExt::filter_map(value, move |item| {
                            let format = format.clone();
                            async move {
                                match format.write(item) {
                                    Ok(value) => Some(value),
                                    Err(error) => {
                                        imports::tracing::error!("Failed to deserialise bytes from stream: {error}");
                                        None
                                    }
                                }
                            }
                        });
                        let value: u32 = manager.outgoing_stream(stream, guard.clone()).await.into();
                        value
                    }; ;
                    self.format.write(value).map_err(|error| { imports::HandleError::InternalError(format!("Error writing response: {error}")) })
                }
                1919010991_u32 => {
                    let (stream, ): (u32,) = self.format.read(&request).map_err(|error| { imports::HandleError::BadPayloadFormat(error.to_string()) })?;
                    let stream = {
                        let value = stream;
                        {
                            let format = format.clone();
                            manager.incoming_stream(imports::StreamID::from(value), guard.clone(),move |bytes| {
                                match imports::ReadFormat::<Foo>::read(&format, &bytes) {
                                    Ok(value) => Some(value),
                                    Err(error) => {
                                        imports::tracing::error!("Failed to deserialise bytes from stream: {error}");
                                        None
                                    }
                                }
                            }).await
                        }
                    };
                    let value = self.server.send(stream).await;
                    let value: () = value;
                    self.format.write(value).map_err(|error| { imports::HandleError::InternalError(format!("Error writing response: {error}")) })
                }
                328483777_u32 => {
                    let (option, tuple, ): (Option<u32>, (Foo, u32,),) = self.format.read(&request).map_err(|error| { imports::HandleError::BadPayloadFormat(error.to_string()) })?;
                    let option = {
                        let value = option;
                        if let Some(value) = value {
                            Some({
                                let format = format.clone();
                                manager.incoming_stream(imports::StreamID::from(value), guard.clone(), move |bytes| {
                                    match imports::ReadFormat::<Foo>::read(&format, &bytes) {
                                        Ok(value) => Some(value),
                                        Err(error) => {
                                            imports::tracing::error!("Failed to deserialise bytes from stream: {error}");
                                            None
                                        }
                                    }
                                }).await
                            })
                        } else { None }
                    };
                    let tuple = {
                        let value = tuple;
                        ({
                             let value = value.0;
                             value
                         }, {
                             let value = value.1;
                             {
                                 let format = format.clone();
                                 manager.incoming_stream(imports::StreamID::from(value), guard.clone(), move |bytes| {
                                     match imports::ReadFormat::<Foo>::read(&format, &bytes) {
                                         Ok(value) => Some(value),
                                         Err(error) => {
                                             imports::tracing::error!("Failed to deserialise bytes from stream: {error}");
                                             None
                                         }
                                     }
                                 }).await
                             }
                         },)
                    };
                    let value = self.server.multiple(option, tuple).await;
                    let value: Result<u32, Error> = match value {
                        Ok(value) => Ok({
                                            let format = format.clone();
                                            let stream = imports::StreamExt::filter_map(value, move |item| {
                                                let format = format.clone();
                                                async move {
                                                    match format.write(item) {
                                                        Ok(value) => Some(value),
                                                        Err(error) => {
                                                            imports::tracing::error!("Failed to deserialise bytes from stream: {error}");
                                                            None
                                                        }
                                                    }
                                                }
                                            });
                                            let value: u32 = manager.outgoing_stream(stream, guard.clone()).await.into();
                                            value
                                        }), Err(value) => Err(value)
                    };
                    self.format.write(value).map_err(|error| { imports::HandleError::InternalError(format!("Error writing response: {error}")) })
                }
                _ => return Err(imports::HandleError::MethodNotFound)
            }
        }
    }
    pub trait StreamsRPCClient<>: Send + Sync {
        type Format: imports::Format;
        #[doc = r""]
        #[doc = r" Returns a future builder that allows certain options to be set before `await`-ing"]
        #[doc = r""]
        #[doc = r" see [`ClientResponseFuture`](imports::ClientResponseFuture)"]
        fn receive(&self) -> imports::ClientResponseFuture<impl imports::Future<Output=Result<imports::IncomingStream<Foo>, imports::ClientError>> + Send>;
        #[doc = r""]
        #[doc = r" Returns a future builder that allows certain options to be set before `await`-ing"]
        #[doc = r""]
        #[doc = r" see [`ClientResponseFuture`](imports::ClientResponseFuture)"]
        fn send(&self, stream: impl imports::Stream<Item=Foo> + Send + 'static) -> imports::ClientResponseFuture<impl imports::Future<Output=Result<(), imports::ClientError>> + Send>;
        #[doc = r""]
        #[doc = r" Returns a future builder that allows certain options to be set before `await`-ing"]
        #[doc = r""]
        #[doc = r" see [`ClientResponseFuture`](imports::ClientResponseFuture)"]
        fn multiple(&self, option: Option<impl imports::Stream<Item=Foo> + Send + 'static>, tuple: (Foo, impl imports::Stream<Item=Foo> + Send + 'static,)) -> imports::ClientResponseFuture<impl imports::Future<Output=Result<Result<imports::IncomingStream<Bar>, Error>, imports::ClientError>> + Send>;
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct StreamsRPCClientImpl<_Format, > {
        client: ::std::sync::Arc::<imports::Client<StreamsRPC<>>>,
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
    impl<_Format, > StreamsRPCClient<> for StreamsRPCClientImpl<_Format, >
    where
        _Format: imports::ReadFormat<u32>,
        _Format: imports::ReadFormat<()>,
        _Format: imports::ReadFormat<Result<u32, Error>>,
        _Format: imports::ReadFormat<Foo>,
        _Format: imports::ReadFormat<Bar>,
        _Format: imports::WriteFormat<()>,
        _Format: imports::WriteFormat<(u32,)>,
        _Format: imports::WriteFormat<(Option<u32>, (Foo, u32,),)>,
        _Format: imports::WriteFormat<Foo>,
        _Format: imports::WriteFormat<Foo>,
        _Format: imports::WriteFormat<Foo>,
    {
        type Format = _Format;
        fn receive(&self) -> imports::ClientResponseFuture<impl Future<Output=Result<imports::IncomingStream<Foo>, imports::ClientError>> + Send> {
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
                    let value: u32 = format.read(&response).map_err(imports::ClientError::Deserialise)?;
                    Ok({
                        let format = format.clone();
                        manager.incoming_stream(imports::StreamID::from(value), guard.clone(), move |bytes| {
                            match imports::ReadFormat::<Foo>::read(&format, &bytes) {
                                Ok(value) => Some(value),
                                Err(error) => {
                                    imports::tracing::error!("Failed to deserialise bytes from stream: {error}");
                                    None
                                }
                            }
                        }).await
                    })
                }
            };
            self.client.send(request, imports::MethodID::_new(4258776482_u32), response)
        }
        fn send(&self, stream: impl imports::Stream<Item=Foo> + Send + 'static) -> imports::ClientResponseFuture<impl Future<Output=Result<(), imports::ClientError>> + Send> {
            let request = {
                let format = self.format.clone();
                let manager = self.client.service_manager();
                let guard = self.client.guard.clone();
                async move {
                    let value = ({
                                     let value = stream;
                                     {
                                         let format = format.clone();
                                         let stream = imports::StreamExt::filter_map(value, move |item| {
                                             let format = format.clone();
                                             async move {
                                                 match format.write(item) {
                                                     Ok(value) => Some(value),
                                                     Err(error) => {
                                                         imports::tracing::error!("Failed to deserialise bytes from stream: {error}");
                                                         None
                                                     }
                                                 }
                                             }
                                         });
                                         let value: u32 = manager.outgoing_stream(stream, guard.clone()).await.into();
                                         value
                                     }
                                 },);
                    format.write(value).map_err(imports::ClientError::Serialise)
                }
            };
            let response = {
                let format = self.format.clone();
                let manager = self.client.service_manager();
                let guard = self.client.guard.clone();
                async move |response: Vec<u8>| -> Result<_, imports::ClientError> {
                    let value: () = format.read(&response).map_err(imports::ClientError::Deserialise)?;
                    Ok(value)
                }
            };
            self.client.send(request, imports::MethodID::_new(1919010991_u32), response)
        }
        fn multiple(&self, option: Option<impl imports::Stream<Item=Foo> + Send + 'static>, tuple: (Foo, impl imports::Stream<Item=Foo> + Send + 'static,)) -> imports::ClientResponseFuture<impl Future<Output=Result<Result<imports::IncomingStream<Bar>, Error>, imports::ClientError>> + Send> {
            let request = {
                let format = self.format.clone();
                let manager = self.client.service_manager();
                let guard = self.client.guard.clone();
                async move {
                    let value = ({
                                     let value = option;
                                     if let Some(value) = value {
                                         Some({
                                                  let format = format.clone();
                                                  let stream = imports::StreamExt::filter_map(value, move |item| {
                                                      let format = format.clone();
                                                      async move {
                                                          match format.write(item) {
                                                              Ok(value) => Some(value),
                                                              Err(error) => {
                                                                  imports::tracing::error!("Failed to deserialise bytes from stream: {error}");
                                                                  None
                                                              }
                                                          }
                                                      }
                                                  });
                                                  let value: u32 = manager.outgoing_stream(stream, guard.clone()).await.into();
                                                  value
                                              })
                                     } else { None }
                                 }, {
                                     let value = tuple;
                                     ({
                                          let value = value.0;
                                          value
                                      }, {
                                          let value = value.1;
                                          {
                                              let format = format.clone();
                                              let stream = imports::StreamExt::filter_map(value, move |item| {
                                                  let format = format.clone();
                                                  async move {
                                                      match format.write(item) {
                                                          Ok(value) => Some(value),
                                                          Err(error) => {
                                                              imports::tracing::error!("Failed to deserialise bytes from stream: {error}");
                                                              None
                                                          }
                                                      }
                                                  }
                                              });
                                              let value: u32 = manager.outgoing_stream(stream, guard.clone()).await.into();
                                              value
                                          }
                                      },)
                                 },);
                    format.write(value).map_err(imports::ClientError::Serialise)
                }
            };
            let response = {
                let format = self.format.clone();
                let manager = self.client.service_manager();
                let guard = self.client.guard.clone();
                async move |response: Vec<u8>| -> Result<_, imports::ClientError> {
                    let value: Result<u32, Error> = format.read(&response).map_err(imports::ClientError::Deserialise)?;
                    Ok(match value {
                        Ok(value) => Ok({
                            let format = format.clone();
                            manager.incoming_stream(imports::StreamID::from(value), guard.clone(), move |bytes| {
                                match imports::ReadFormat::<Bar>::read(&format, &bytes) {
                                    Ok(value) => Some(value),
                                    Err(error) => {
                                        imports::tracing::error!("Failed to deserialise bytes from stream: {error}");
                                        None
                                    }
                                }
                            }).await
                        }),
                        Err(value) => Err(value)
                    })
                }
            };
            self.client.send(request, imports::MethodID::_new(328483777_u32), response)
        }
    }
}