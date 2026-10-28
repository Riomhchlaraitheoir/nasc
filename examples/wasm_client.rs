use nasc::connection::ConnectionOptions;
use nasc::format::json::Json;
use nasc::websocket::browser::new_websocket_transport;

include!("traits/todo.rs");

fn main() {
    wasm_bindgen_futures::spawn_local(async move {
        let client = new_websocket_transport::<TodoService, _>(
            "http://localhost:8080/api/todo",
            Json,
            || {},
            ConnectionOptions::default(),
        )
        .await
        .expect("Failed to connect");
        for todo in client.get_todos().await.expect("get_todos failed") {
            println!("{todo:?}");
        }
        if let Some(todo) = client
            .get_todo("next".to_string())
            .await
            .expect("get_todo failed")
        {
            println!("{todo:?}");
        }
        client
            .new_todo(Todo {
                name: "Some task".to_string(),
                description: "A description of the task".to_string(),
            })
            .await
            .expect("new_todo failed");
    });
}
