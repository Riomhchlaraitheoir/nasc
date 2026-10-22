//! Defines the [`Spawner`] trait which defines how to spawn a task for your chosen async executor

use std::sync::Arc;
use async_executor::Executor;
use futures::future::Abortable;

/// Defines an async executor capable of spawning futures
pub trait Spawner: Send + Sync {
    /// Spawn the given future
    fn spawn(&self, f: Abortable<impl Future<Output = ()> + Send + 'static>);
}

impl<S: Spawner> Spawner for Arc<S> {
    fn spawn(&self, f: Abortable<impl Future<Output = ()> + Send + 'static>) {
        (**self).spawn(f);
    }
}

/// Spawner implementation for tokio
#[cfg(feature = "tokio")]
pub struct TokioSpawner;

#[cfg(feature = "tokio")]
impl Spawner for TokioSpawner {
    fn spawn(&self, f: Abortable<impl Future<Output = ()> + Send + 'static>) {
        tokio::spawn(f);
    }
}

/// Spawner implementation for browser
#[cfg(feature = "browser-websocket")]
pub struct BrowserSpawner;

#[cfg(feature = "browser-websocket")]
impl Spawner for BrowserSpawner {
    fn spawn(&self, f: Abortable<impl Future<Output = ()> + Send + 'static>) {
        wasm_bindgen_futures::spawn_local(f);
    }
}

impl Spawner for Executor<'_> {
    fn spawn(&self, f: Abortable<impl Future<Output = ()> + Send + 'static>) {
        self.spawn(f).detach();
    }
}