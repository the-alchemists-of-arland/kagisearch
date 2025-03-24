use std::time::Duration;

pub(crate) const HOST: &str = "https://kagi.com";
pub(crate) const MAX_RETRIES: u32 = 5;
pub(crate) const RETRY_TIMEOUT: Duration = Duration::from_millis(1000);
