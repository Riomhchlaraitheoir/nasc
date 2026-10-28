use std::fmt::Debug;
use nasc::service;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct Book {
    pub id: u64,
    pub title: String,
    pub author: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct Author {
    pub id: u64,
    pub name: String,
    pub quote: String,
}

#[service]
pub trait ResourcesService {
    fn books() -> impl Service<ResourceService<Book>>;
    fn authors() -> impl Service<ResourceService<Author>>;
}

#[service]
pub trait ResourceService<T: Resource> {
    fn subscribe(listener: impl Service<Subscriber<T>>);
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
