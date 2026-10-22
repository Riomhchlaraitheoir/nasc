//! Defines the [`AsyncRuntime`] trait which defines how to spawn a task for your chosen async executor

use std::time::Duration;

/// Defines an async executor capable of spawning futures
pub trait AsyncRuntime: Clone + Send + Sync + 'static {
    /// A future that sleeps for a predefined duration
    type SleepFuture: Future<Output = ()> + Unpin + Send + 'static;

    /// Spawn the given future
    fn spawn(&self, f: impl Future<Output = ()> + Send + 'static);

    /// Return a future that sleeps for the given duration
    fn sleep(&self, duration: Duration) -> Self::SleepFuture;
}

/// Spawner implementation for tokio
#[cfg(feature = "tokio")]
#[derive(Debug, Clone)]
pub struct TokioRuntime;

#[cfg(feature = "tokio")]
impl AsyncRuntime for TokioRuntime {
    type SleepFuture = std::pin::Pin<Box<tokio::time::Sleep>>;
    fn spawn(&self, f: impl Future<Output = ()> + Send + 'static) {
        tokio::spawn(f);
    }
    fn sleep(&self, duration: Duration) -> Self::SleepFuture {
        Box::pin(tokio::time::sleep(duration))
    }
}

/// Spawner implementation for browser
#[cfg(feature = "browser-websocket")]
#[derive(Debug, Clone)]
pub struct BrowserRuntime;

#[cfg(feature = "browser-websocket")]
impl AsyncRuntime for BrowserRuntime {
    type SleepFuture = send_wrapper::SendWrapper<gloo_timers::future::TimeoutFuture>;
    fn spawn(&self, f: impl Future<Output = ()> + Send + 'static) {
        wasm_bindgen_futures::spawn_local(f);
    }

    fn sleep(&self, duration: Duration) -> Self::SleepFuture {
        send_wrapper::SendWrapper::new(gloo_timers::future::TimeoutFuture::new(u32::try_from(duration.as_millis()).expect("Durations over 2^32ms are not supported in wasm environments")))
    }
}
