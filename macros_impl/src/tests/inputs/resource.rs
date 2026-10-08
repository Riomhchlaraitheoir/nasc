#[rpc]
pub trait Resources<T: Debug> {
    fn subscribe() -> impl Stream<Item=T>;
    fn list() -> Vec<T>;
    fn get(id: u64) -> Option<T>;
    fn new(value: T);
}