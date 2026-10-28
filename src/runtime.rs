//! Defines the [`AsyncRuntime`] trait which defines how to spawn a task for your chosen async executor

use std::panic::RefUnwindSafe;
use futures::future::BoxFuture;

/// Defines an async executor capable of spawning futures
pub trait AsyncRuntime: Send + Sync + RefUnwindSafe + 'static {

    /// Spawn the given future
    fn spawn(&self, f: BoxFuture<'static, ()>);
}

#[cfg(feature = "tokio")]
#[derive(Debug, Clone)]
/// Runtime for use with tokio, spawns tasks using [`tokio::spawn`]
pub struct TokioRuntime;

#[cfg(feature = "tokio")]
impl AsyncRuntime for TokioRuntime {
    fn spawn(&self, f: BoxFuture<'static, ()>) {
        tokio::spawn(f);
    }
}

/// Spawner implementation for browser
#[cfg(feature = "browser-websocket")]
#[derive(Debug, Clone)]
pub struct BrowserRuntime;

#[cfg(feature = "browser-websocket")]
impl AsyncRuntime for BrowserRuntime {
    fn spawn(&self, f: BoxFuture<'static, ()>) {
        wasm_bindgen_futures::spawn_local(f);
    }
}
