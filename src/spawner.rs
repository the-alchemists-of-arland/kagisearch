/// Spawner trait
/// Used to spawn futures
/// This is required because the browser handler runs in a separate thread
/// and we need to spawn the handler in the same runtime as the browser
pub trait Spawner {
    fn spawn(future: impl Future<Output = ()> + Send + 'static);
}

#[cfg(feature = "tokio-runtime")]
/// Implementation of the Spawner trait for tokio runtime
impl Spawner for tokio::runtime::Handle {
    fn spawn(future: impl Future<Output = ()> + Send + 'static) {
        tokio::runtime::Handle::current().spawn(future);
    }
}

#[cfg(feature = "async-std-runtime")]
/// Implementation of the Spawner trait for async-std runtime
impl Spawner for async_std::task::JoinHandle<()> {
    fn spawn(future: impl Future<Output = ()> + Send + 'static) {
        async_std::task::spawn(future);
    }
}
