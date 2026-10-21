#![doc = include_str!("./examples.md")]

use trait_rpc::format::json::Json;
use trait_rpc::{Rpc};
use trait_rpc::websocket::tokio::new_websocket_transport;

include!("traits/todo.rs");

#[tokio::main]
async fn main() {
    let client = {
        let client = new_websocket_transport("ws://localhost:8080/api/todo", Json, ||{
            println!("Client closed");
        }).await.unwrap();
        TodoService::client(client, Json)
    };
    for todo in client.get_todos().await.expect("get_todos failed") {
        println!("{todo:?}");
    }
    if let Some(todo) = client.get_todo("next".to_string()).await.expect("get_todo failed") {
        println!("{todo:?}");
    }
    client.new_todo(Todo {
        name: "Some task".to_string(),
        description: "A description of the task".to_string(),
    }).await.expect("new_todo failed");
}