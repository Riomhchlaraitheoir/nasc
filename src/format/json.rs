//! Provides support for the JSON ([JavaScript Object Notation](https://www.json.org/)) format

use crate::format::{Format, ReadFormat, WriteFormat};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::error::Error;

const CONTENT_TYPE: &str = "application/json";

#[derive(Debug, Copy, Clone)]
/// [JavaScript Object Notation](https://www.json.org/)
pub struct Json;

impl Format for Json {
    fn content_type(&self) -> &'static str {
        CONTENT_TYPE
    }
    fn subprotocol(&self) -> &'static str {
        "json"
    }
}

impl<Read> ReadFormat<Read> for Json
where
    Read: DeserializeOwned,
{
    fn read(&self, reader: &[u8]) -> Result<Read, Box<dyn Error + Send>> {
        serde_json::from_slice(reader).map_err(|error| Box::new(error) as Box<dyn Error + Send>)
    }
}

impl<Write> WriteFormat<Write> for Json
where
    Write: Serialize,
{
    fn write(&self, value: Write) -> Result<Vec<u8>, Box<dyn Error + Send>> {
        serde_json::to_vec(&value).map_err(|error| Box::new(error) as Box<dyn Error + Send>)
    }
}
