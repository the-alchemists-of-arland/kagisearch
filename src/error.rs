use chromiumoxide::error::CdpError;

#[macro_export]
macro_rules! auth_error {
    ($($arg:tt)*) => {
        Error::AuthError(format!($($arg)*))
    }
}

#[macro_export]
macro_rules! element_error {
    ($($arg:tt)*) => {
        Error::ElementNotFound(format!($($arg)*))
    }
}

#[macro_export]
macro_rules! browser_error {
    ($($arg:tt)*) => {
        Error::BrowserError(format!($($arg)*))
    }
}

#[macro_export]
macro_rules! spawner_error {
    ($($arg:tt)*) => {
        Error::SpawnerError(format!($($arg)*))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Authentication failed: {0}")]
    AuthError(String),
    #[error("Element not found: {0}")]
    ElementNotFound(String),
    #[error("Browser error: {0}")]
    BrowserError(String),
    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("CDP error: {0}")]
    CdpError(#[from] CdpError),
    #[error("Spawner error: {0}")]
    SpawnerError(String),
}
