#[cfg(feature = "tokio-runtime")]
pub(crate) fn spawn(future: impl futures::Future<Output = ()> + Send + 'static) {
    tokio::spawn(future);
}
#[cfg(feature = "async-std-runtime")]
pub(crate) fn spawn(future: impl futures::Future<Output = ()> + Send + 'static) {
    async_std::task::spawn(future);
}
