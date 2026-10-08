use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        nosend: { feature = "single-thread" },
        send: { not(nosend) },
        browser: { all(feature = "browser", target_arch = "wasm32") },
        tokio_websocket: { all(feature = "tokio_tungstenite", not(target_arch = "wasm32")) },
        wasm_websocket: { all(feature = "browser", target_arch = "wasm32") },
    }
}