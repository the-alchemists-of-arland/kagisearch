use std::sync::Arc;

use chromiumoxide::{Browser, cdp::browser_protocol::browser::BrowserContextId};
use tracing::debug;

use crate::spawner::spawn;

pub(crate) struct Page {
    page: chromiumoxide::Page,
    context_id: Option<BrowserContextId>,
    #[cfg(feature = "tokio-runtime")]
    browser: Arc<tokio::sync::RwLock<Browser>>,
    #[cfg(feature = "async-std-runtime")]
    browser: Arc<async_std::sync::RwLock<Browser>>,
}

impl Page {
    pub fn new(
        page: chromiumoxide::Page,
        context_id: Option<BrowserContextId>,
        #[cfg(feature = "tokio-runtime")] browser: Arc<tokio::sync::RwLock<Browser>>,
        #[cfg(feature = "async-std-runtime")] browser: Arc<async_std::sync::RwLock<Browser>>,
    ) -> Self {
        Self {
            page,
            context_id,
            browser,
        }
    }

    pub fn inner(&self) -> &chromiumoxide::Page {
        &self.page
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        if let Some(context_id) = self.context_id.take() {
            let browser = self.browser.clone();
            debug!("Disposing browser context: {:?}", context_id);
            spawn(async move {
                if let Err(e) = browser
                    .read()
                    .await
                    .dispose_browser_context(context_id)
                    .await
                {
                    debug!("Failed to dispose browser context: {:?}", e);
                }
            });
        }
    }
}
