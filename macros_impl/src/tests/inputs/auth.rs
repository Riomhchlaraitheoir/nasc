
#[service]
trait PublicApi {
    async fn login(email: String, password: String) -> Result<(AuthToken, impl Service<AuthApi>), LoginError>;
}

#[service]
trait AuthApi {
    fn private_fn();
}
