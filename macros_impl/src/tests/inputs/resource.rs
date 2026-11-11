
#[service]
pub trait ResourcesService {
    fn books() -> impl Service<ResourceService<Book>>;
    fn authors() -> impl Service<ResourceService<Author>>;
}

#[service]
pub trait ResourceService<T: Resource> {
    fn subscribe(listener: impl Service<Subscriber<T>>);
    fn updates() ->impl Stream<Item=T>;
    fn list() -> Vec<T>;
    fn get(id: u64) -> Option<T>;
    fn new(value: T);
}

#[service]
pub trait Subscriber<T: Resource> {
    fn deleted(value: T);
    fn updated(value: T);
    fn new(value: T);
}
