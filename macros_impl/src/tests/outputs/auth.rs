#[allow(
    unused_imports,
    reason = "These might not always be used, but they should be available in this module anyway"
)]
use public_api::{
    PublicApi, PublicApiClient, PublicApiClientImpl, PublicApiHandler, PublicApiServer,
};
#[allow(clippy::unused_unit)]
#[allow(clippy::multiple_bound_locations)]
#[allow(
    unused_imports,
    reason = "These might not always be used, but it's easier to include always"
)]
mod public_api {
    use super::*;
    use ::nasc::__macros::{imports, traits::*};
    #[doc = r""]
    #[doc = r" This is the [`Service`](imports::Service) definition for this service"]
    pub struct PublicApi(imports::PhantomData<fn() -> ()>);
    impl PublicApi {
        pub const METHOD_ID_LOGIN: imports::MethodID = imports::MethodID::_new(3380555570_u32);
    }
    impl imports::Service for PublicApi {
        type Client<_Format: imports::Format> = PublicApiClientImpl<_Format>;
        fn client<_Format: imports::Format>(
            client: imports::Client<Self>,
            format: _Format,
        ) -> Self::Client<_Format> {
            PublicApiClientImpl {
                client,
                format,
                _generics: imports::PhantomData,
            }
        }
        fn service_name() -> &'static str {
            stringify!(PublicApi)
        }
    }
    impl<_Server, _Format> imports::ServiceWithServer<_Server, _Format> for PublicApi
    where
        _Format: imports::Format,
        PublicApiHandler<_Server, _Format>: imports::Handler,
        _Server: PublicApiServer<_Format>,
    {
        type Handler = PublicApiHandler<_Server, _Format>;
        fn handler(server: _Server, format: _Format) -> Self::Handler {
            PublicApiHandler {
                server,
                format,
                _generics: imports::PhantomData,
            }
        }
    }
    #[doc = r""]
    #[doc = r" This is the trait which is used by the server side in order to serve the client's"]
    #[doc = r" requests"]
    pub trait PublicApiServer<_Format>: Send + Sync + 'static
    where
        _Format: imports::Format,
    {
        fn login(
            &self,
            email: String,
            password: String,
        ) -> impl imports::Future<
            Output = Result<(AuthToken, impl imports::IntoHandler<AuthApi, _Format>), LoginError>,
        > + Send;
    }
    #[doc = " A [Handler](imports::Handler) which handles requests/responses for a given service"]
    #[derive(Debug, Clone)]
    pub struct PublicApiHandler<_Server, _Format> {
        server: _Server,
        format: _Format,
        _generics: imports::PhantomData<fn() -> ()>,
    }
    impl<_Server, _Format> imports::Handler for PublicApiHandler<_Server, _Format>
    where
        _Server: PublicApiServer<_Format>,
        _Format: imports::ReadFormat<(String, String)>,
        _Format: imports::WriteFormat<Result<(AuthToken, u32), LoginError>>,
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
                3380555570_u32 => {
                    let (email, password): (String, String) =
                        self.format.read(&request).map_err(|error| {
                            imports::HandleError::BadPayloadFormat(error.to_string())
                        })?;
                    let email = {
                        let value = email;
                        value
                    };
                    let password = {
                        let value = password;
                        value
                    };
                    let value = self.server.login(email, password).await;
                    let value: Result<(AuthToken, u32), LoginError> = match value {
                        Ok(value) => Ok((
                            {
                                let value = value.0;
                                value
                            },
                            {
                                let value = value.1;
                                manager
                                    .outgoing_service(
                                        value.into_handler(format.clone()),
                                        guard.clone(),
                                    )
                                    .await
                                    .as_u32()
                            },
                        )),
                        Err(value) => Err(value),
                    };
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
    pub trait PublicApiClient: Send + Sync {
        type Format: imports::Format;
        #[doc = r""]
        #[doc = r" Returns a future builder that allows certain options to be set before `await`-ing"]
        #[doc = r""]
        #[doc = r" see [`ClientResponseFuture`](imports::ClientResponseFuture)"]
        fn login(
            &self,
            email: String,
            password: String,
        ) -> imports::ClientResponseFuture<
            impl imports::Future<
                Output = Result<
                    Result<
                        (
                            AuthToken,
                            <AuthApi as imports::Service>::Client<Self::Format>,
                        ),
                        LoginError,
                    >,
                    imports::ClientError,
                >,
            > + Send,
        >;
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct PublicApiClientImpl<_Format> {
        client: imports::Client<PublicApi>,
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
    impl<_Format> PublicApiClient for PublicApiClientImpl<_Format>
    where
        _Format: imports::ReadFormat<Result<(AuthToken, u32), LoginError>>,
        _Format: imports::WriteFormat<(String, String)>,
    {
        type Format = _Format;
        fn login(
            &self,
            email: String,
            password: String,
        ) -> imports::ClientResponseFuture<
            impl Future<
                Output = Result<
                    Result<
                        (
                            AuthToken,
                            <AuthApi as imports::Service>::Client<Self::Format>,
                        ),
                        LoginError,
                    >,
                    imports::ClientError,
                >,
            > + Send,
        > {
            let request = {
                let format = self.format.clone();
                let manager = self.client.service_manager();
                let guard = self.client.guard.clone();
                async move {
                    let value = (
                        {
                            let value = email;
                            value
                        },
                        {
                            let value = password;
                            value
                        },
                    );
                    format.write(value).map_err(imports::ClientError::Serialise)
                }
            };
            let response = {
                let format = self.format.clone();
                let manager = self.client.service_manager();
                let guard = self.client.guard.clone();
                async move |response: Vec<u8>| -> Result<_, imports::ClientError> {
                    let value: Result<(AuthToken, u32), LoginError> = format
                        .read(&response)
                        .map_err(imports::ClientError::Deserialise)?;
                    Ok(match value {
                        Ok(value) => Ok((
                            {
                                let value = value.0;
                                value
                            },
                            {
                                let value = value.1;
                                manager
                                    .incoming_service::<AuthApi, _>(
                                        imports::ServiceID::from(value),
                                        guard.clone(),
                                        format.clone(),
                                    )
                                    .await
                            },
                        )),
                        Err(value) => Err(value),
                    })
                }
            };
            self.client
                .send(request, imports::MethodID::_new(3380555570_u32), response)
        }
    }
}
#[allow(
    unused_imports,
    reason = "These might not always be used, but they should be available in this module anyway"
)]
use auth_api::{AuthApi, AuthApiClient, AuthApiClientImpl, AuthApiHandler, AuthApiServer};

#[allow(clippy::unused_unit)]
#[allow(clippy::multiple_bound_locations)]
#[allow(
    unused_imports,
    reason = "These might not always be used, but it's easier to include always"
)]
mod auth_api {
    use super::*;
    use ::nasc::__macros::{imports, traits::*};
    #[doc = r""]
    #[doc = r" This is the [`Service`](imports::Service) definition for this service"]
    pub struct AuthApi(imports::PhantomData<fn() -> ()>);
    impl AuthApi {
        pub const METHOD_ID_PRIVATE_FN: imports::MethodID = imports::MethodID::_new(2025707977_u32);
    }
    impl imports::Service for AuthApi {
        type Client<_Format: imports::Format> = AuthApiClientImpl<_Format>;
        fn client<_Format: imports::Format>(
            client: imports::Client<Self>,
            format: _Format,
        ) -> Self::Client<_Format> {
            AuthApiClientImpl {
                client,
                format,
                _generics: imports::PhantomData,
            }
        }
        fn service_name() -> &'static str {
            stringify!(AuthApi)
        }
    }
    impl<_Server, _Format> imports::ServiceWithServer<_Server, _Format>
        for AuthApi
    where
        _Format: imports::Format,
        AuthApiHandler<_Server, _Format>: imports::Handler,
        _Server: AuthApiServer<_Format>
    {
        type Handler = AuthApiHandler<_Server, _Format>;
        fn handler(server: _Server, format: _Format) -> Self::Handler {
            AuthApiHandler {
                server,
                format,
                _generics: imports::PhantomData,
            }
        }
    }
    #[doc = r""]
    #[doc = r" This is the trait which is used by the server side in order to serve the client's"]
    #[doc = r" requests"]
    pub trait AuthApiServer<_Format>: Send + Sync + 'static
    where
        _Format: imports::Format
    {
        fn private_fn(&self) -> impl imports::Future<Output = ()> + Send;
    }
    #[doc = " A [Handler](imports::Handler) which handles requests/responses for a given service"]
    #[derive(Debug, Clone)]
    pub struct AuthApiHandler<_Server, _Format> {
        server: _Server,
        format: _Format,
        _generics: imports::PhantomData<fn() -> ()>,
    }
    impl<_Server, _Format> imports::Handler for AuthApiHandler<_Server, _Format>
    where
        _Server: AuthApiServer<_Format>,
        _Format: imports::ReadFormat<()>,
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
                2025707977_u32 => {
                    let (): () = self.format.read(&request).map_err(|error| {
                        imports::HandleError::BadPayloadFormat(error.to_string())
                    })?;
                    let value = self.server.private_fn().await;
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
    pub trait AuthApiClient: Send + Sync {
        type Format: imports::Format;
        #[doc = r""]
        #[doc = r" Returns a future builder that allows certain options to be set before `await`-ing"]
        #[doc = r""]
        #[doc = r" see [`ClientResponseFuture`](imports::ClientResponseFuture)"]
        fn private_fn(
            &self,
        ) -> imports::ClientResponseFuture<
            impl imports::Future<Output = Result<(), imports::ClientError>> + Send,
        >;
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct AuthApiClientImpl<_Format> {
        client: imports::Client<AuthApi>,
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
    impl<_Format> AuthApiClient for AuthApiClientImpl<_Format>
    where
        _Format: imports::ReadFormat<()>,
        _Format: imports::WriteFormat<()>,
    {
        type Format = _Format;
        fn private_fn(
            &self,
        ) -> imports::ClientResponseFuture<
            impl Future<Output = Result<(), imports::ClientError>> + Send,
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
                    let value: () = format
                        .read(&response)
                        .map_err(imports::ClientError::Deserialise)?;
                    Ok(value)
                }
            };
            self.client
                .send(request, imports::MethodID::_new(2025707977_u32), response)
        }
    }
}
