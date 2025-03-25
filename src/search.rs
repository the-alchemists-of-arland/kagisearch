use std::sync::Arc;

use chromiumoxide::{
    BrowserConfig, Element,
    browser::Browser,
    cdp::browser_protocol::{
        network::{Cookie, CookieParam},
        target::{CreateBrowserContextParams, CreateTargetParams},
    },
    handler::viewport::Viewport,
};
use futures::StreamExt;
use futures_timer::Delay;
use tracing::debug;
use url::Url;

use crate::{
    Error,
    auth::{handle_signin, handle_token},
    auth_error, browser_error,
    consts::{HOST, MAX_RETRIES, RETRY_TIMEOUT},
    page::Page,
    spawner::spawn,
};

/// Search result
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug)]
pub struct SearchResult {
    /// Title of the search result
    pub title: String,
    /// URL of the search result
    pub url: String,
    /// Snippet of the search result
    pub snippet: String,
}

/// Authentication type
#[derive(Debug)]
pub enum AuthType {
    /// Login with username, password and optional 2FA code
    Login(String, String, Option<String>),
    /// Login with a token
    Token(String),
    /// Load cookies
    Cookies(Vec<CookieParam>),
    /// Use incognito mode
    Icognito,
}

/// Browser instance
pub struct Kagi {
    auth_type: AuthType,
    #[cfg(feature = "tokio-runtime")]
    browser: Arc<tokio::sync::RwLock<Browser>>,
    #[cfg(feature = "async-std-runtime")]
    browser: Arc<async_std::sync::RwLock<Browser>>,
}

impl Kagi {
    /// Create a new browser instance with authentication.
    ///
    /// This method initializes a new headless browser instance with pre-configured settings optimized
    /// for web scraping. It requires a spawner implementation to handle browser events in the background.
    ///
    /// # Authentication Types
    ///
    /// The browser can be authenticated in three ways:
    /// - Using email and password (with optional 2FA)
    /// - Using a Kagi login token
    /// - Using pre-saved cookies
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Browser initialization fails
    /// - Cookie loading fails (when using `AuthType::Cookies`)
    ///
    pub async fn new(auth_type: AuthType) -> Result<Self, Error> {
        let viewport = Viewport {
            width: 1920,
            height: 1080,
            ..Default::default()
        };
        let (browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                .viewport(viewport)
                .incognito()
                .args([
                    "--disable-blink-features=AutomationControlled",
                    "--disable-features=IsolateOrigins,site-per-process",
                    "--disable-site-isolation-trials",
                    "--no-sandbox",
                    "--disable-setuid-sandbox",
                    "--disable-dev-shm-usage",
                    "--disable-accelerated-2d-canvas",
                    "--no-first-run",
                    "--no-zygote",
                    "--disable-gpu",
                    "--hide-scrollbars",
                    "--mute-audio",
                    "--disable-background-networking",
                    "--disable-background-timer-throttling",
                    "--disable-backgrounding-occluded-windows",
                    "--disable-breakpad",
                    "--disable-component-extensions-with-background-pages",
                    "--disable-extensions",
                    "--disable-features=TranslateUI",
                    "--disable-ipc-flooding-protection",
                    "--disable-renderer-backgrounding",
                    "--enable-features=NetworkService,NetworkServiceInProcess",
                    "--force-color-profile=srgb",
                    "--metrics-recording-only",
                ])
                .build()
                .map_err(|e| browser_error!("{}", e))?,
        )
        .await?;
        spawn(async move {
            while let Some(h) = handler.next().await {
                match h {
                    Ok(_) => continue,
                    Err(e) => {
                        debug!("Browser handler error: {}", e);
                        if e.to_string().contains("Browser closed") {
                            break;
                        }
                    }
                }
            }
            debug!("Browser handler stopped");
        });
        if let AuthType::Cookies(cookies) = &auth_type {
            browser.set_cookies(cookies.to_vec()).await?;
            debug!("Cookies loaded");
        }
        Ok(Self {
            auth_type,
            #[cfg(feature = "tokio-runtime")]
            browser: Arc::new(tokio::sync::RwLock::new(browser)),
            #[cfg(feature = "async-std-runtime")]
            browser: Arc::new(async_std::sync::RwLock::new(browser)),
        })
    }

    /// Close the browser instance
    pub async fn close(&self) -> Result<(), Error> {
        let mut browser = self.browser.write().await;
        browser.close().await?;
        browser.wait().await?;
        Ok(())
    }

    /// Get the cookies stored in the browser context
    pub async fn cookies(&self) -> Result<Vec<Cookie>, Error> {
        let cookies = self.browser.read().await.get_cookies().await?;
        Ok(cookies)
    }

    /// Initialize a new page with optional incognito mode
    async fn init_page(&self, auth_type: &AuthType) -> Result<Page, Error> {
        let browser = self.browser.read().await;
        let context_id = if let AuthType::Icognito = auth_type {
            debug!("Creating incognito browser context");
            Some(
                browser
                    .create_browser_context(CreateBrowserContextParams::default())
                    .await?,
            )
        } else {
            None
        };
        let mut builder = CreateTargetParams::builder().url("about:blank");
        if let Some(context_id) = &context_id {
            builder = builder.browser_context_id(context_id.clone());
        }
        let param = builder.build().map_err(|e| browser_error!("{}", e))?;
        let page = browser.new_page(param).await?;
        let _ = browser;
        Ok(Page::new(page, context_id, self.browser.clone()))
    }

    /// Performs a search query on Kagi and returns the results.
    ///
    /// This method will:
    /// 1. Create a new page
    /// 2. Navigate to Kagi search
    /// 3. Handle authentication if needed
    /// 4. Extract search results
    ///
    /// # Parameters
    ///
    /// - `query`: The search term to look for
    /// - `limit`: Maximum number of results to return
    /// - `auth_type`: Optional authentication type to use for this search performing in incognito
    ///     mode. Only `AuthType::Token` and `AuthType::Login` are supported.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(Vec<SearchResult>))` if the search was successful and results were found.
    /// Returns `Ok(None)` if no results were found.
    /// Returns `Err` if any error occurred during the search process.
    ///
    /// Each `SearchResult` contains:
    /// - `title`: The title of the search result
    /// - `url`: The URL of the result
    /// - `snippet`: A brief description or snippet from the result
    ///
    /// # Examples
    ///
    /// ```rust
    /// use kagisearch::{Kagi, AuthType};
    ///
    /// #[cfg_attr(feature = "tokio-runtime", tokio::main)]
    /// #[cfg_attr(feature = "async-std-runtime", async_std::main)]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let token = std::env::var("KAGI_TOKEN")?;
    ///     #[cfg(feature = "tokio-runtime")]
    ///     let mut kagi = Kagi::new(AuthType::Token(token)).await?;
    ///     #[cfg(feature = "async-std-runtime")]
    ///     let mut kagi = Kagi::new(AuthType::Token(token)).await?;
    ///     
    ///     // Search for "Rust programming" and get up to 5 results
    ///     let results = kagi.search("Rust programming", 5, None).await?;
    ///     
    ///     if let Some(results) = results {
    ///         for result in results {
    ///             println!("Title: {}", result.title);
    ///             println!("URL: {}", result.url);
    ///             println!("Snippet: {}", result.snippet);
    ///         }
    ///     }
    ///     
    ///     // Don't forget to close the browser
    ///     kagi.close().await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Page initialization fails
    /// - Navigation to search page fails
    /// - Authentication fails
    /// - Result extraction fails
    ///
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        auth_type: Option<AuthType>,
    ) -> Result<Option<Vec<SearchResult>>, Error> {
        let page = self.init_page(&self.auth_type).await?;

        let auth_type = if let Some(auth_type) = &auth_type {
            auth_type
        } else {
            &self.auth_type
        };

        loop {
            let url = Url::parse_with_params(&format!("{}/search", HOST), &[("q", query)])?;
            page.inner().goto(url).await?.wait_for_navigation().await?;
            let Some(url) = page.inner().url().await? else {
                return Err(browser_error!("Failed to get URL"));
            };
            let url = Url::parse(&url)?;
            if url.path() == "/signin" {
                debug!("Sign in required");
                match auth_type {
                    AuthType::Login(email, password, code) => {
                        handle_signin(page.inner(), email, password, code.as_deref()).await?;
                    }
                    AuthType::Token(token) => {
                        handle_token(page.inner(), token).await?;
                    }
                    AuthType::Cookies(_) => {
                        return Err(auth_error!("Invalid cookies"));
                    }
                    AuthType::Icognito => {
                        return Err(auth_error!("Incognito mode not supported"));
                    }
                }
                continue;
            }
            if url.path() != "/search" {
                return Err(browser_error!("Failed to navigate to search page"));
            }
            debug!("Already signed in");
            break;
        }

        let search_results = async {
            for _ in 0..MAX_RETRIES {
                let results = page.inner().find_element(".results-box").await?;
                debug!("Results found");
                let search_results = results.find_elements(".search-result").await?;
                debug!("Search results found");
                if search_results.is_empty() {
                    debug!("No search results found, waiting");
                    // Sometimes the results take a while to load
                    Delay::new(RETRY_TIMEOUT).await;
                    continue;
                }
                return Ok(Some(search_results));
            }
            Ok::<Option<Vec<Element>>, Error>(None)
        }
        .await?;

        let Some(search_results) = search_results else {
            return Ok(None);
        };

        let mut results = Vec::new();
        for result in &search_results {
            if results.len() >= limit {
                break;
            }
            let Ok(title) = result.find_element(".__sri-title").await else {
                debug!("Title class not found");
                continue;
            };
            let Some(title) = title.inner_text().await? else {
                debug!("Title class not found");
                continue;
            };
            let Ok(url) = result.find_element(".__sri-url-box").await else {
                debug!("URL class not found");
                continue;
            };
            let Some(url) = url.find_element("a").await?.attribute("href").await? else {
                debug!("URL attribute not found");
                continue;
            };
            let Ok(snippet) = result.find_element(".__sri-desc").await else {
                debug!("Description class not found");
                continue;
            };
            let Some(snippet) = snippet.inner_text().await? else {
                debug!("Description class not found");
                continue;
            };
            results.push(SearchResult {
                title,
                url,
                snippet,
            });
        }
        Ok(Some(results))
    }
}
