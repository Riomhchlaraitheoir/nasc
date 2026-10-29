use macros::service;

#[allow(dead_code, reason = "only an example")]
pub struct AuthToken(String);
pub struct LoginError;

#[service]
trait PublicApi {
    async fn login(email: String, password: String) -> Result<(AuthToken, impl Service<AuthApi>), LoginError>;
}

#[service]
trait AuthApi {
    fn private_fn();
}

fn main() {

}