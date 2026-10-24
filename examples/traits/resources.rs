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
pub trait Resources<T: Debug> {
    fn subscribe() -> impl Stream<Item=T>;
    fn list() -> Vec<T>;
    fn get(id: u64) -> Option<T>;
    fn new(value: T);
}
