use playwright::{
    Playwright,
    api::{BrowserContext, Cookie, Page, frame::FrameState},
};
use tracing::debug;
use url::Url;

use crate::{
    Error,
    auth::{handle_signin, handle_token},
    auth_error,
    consts::{BROWSER_INIT_SCRIPT, HOST, SCREEN_INIT_SCRIPT},
    element_error,
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
pub enum AuthType {
    /// Login with username, password and optional 2FA code
    Login(String, String, Option<String>),
    /// Login with a token
    Token(String),
    /// Load cookies
    Cookies(Vec<Cookie>),
}

/// Browser instance
pub struct Browser {
    context: BrowserContext,
    auth_type: AuthType,

    // Store these to prevent them from being dropped
    _playwright: Playwright,
    _browser: playwright::api::Browser,
}

impl Browser {
    /// Create a new browser instance
    pub async fn new(auth_type: AuthType) -> Result<Self, Error> {
        let _playwright = Playwright::initialize().await?;
        _playwright.prepare()?; // Install browsers
        let chromium = _playwright.chromium();
        let _browser = chromium
            .launcher()
            .headless(true)
            .args(
                &[
                    "--disable-blink-features=AutomationControlled",
                    "--disable-features=IsolateOrigins,site-per-process",
                    "--disable-site-isolation-trials",
                    "--disable-web-security",
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
                ]
                .map(Into::into),
            )
            .launch()
            .await?;
        let context = _browser.context_builder().build().await?;
        context.add_init_script(BROWSER_INIT_SCRIPT).await?;
        if let AuthType::Cookies(cookies) = &auth_type {
            context.add_cookies(cookies).await?;
            debug!("Cookies loaded");
        }
        Ok(Self {
            context,
            auth_type,
            _playwright,
            _browser,
        })
    }

    /// Get the cookies stored in the browser context
    pub async fn cookies(&self) -> Result<Option<Vec<Cookie>>, Error> {
        let storage = self.context.storage_state().await?;
        Ok(storage.cookies)
    }

    /// Initialize a new page with anti-detection scripts
    async fn init_page(&self) -> Result<Page, Error> {
        let page = self.context.new_page().await?;
        page.add_init_script(SCREEN_INIT_SCRIPT).await?;
        Ok(page)
    }

    /// Search for a query and return the results
    /// The limit parameter specifies the maximum number of results to return
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, Error> {
        let page = self.init_page().await?;

        loop {
            let url = Url::parse_with_params(&format!("{}/search", HOST), &[("q", query)])?;
            page.goto_builder(url.as_str())
                .wait_until(playwright::api::DocumentLoadState::NetworkIdle)
                .goto()
                .await?;

            if Url::parse(&page.url()?)?.path() == "/signin" {
                debug!("Sign in required");
                match &self.auth_type {
                    AuthType::Login(email, password, code) => {
                        handle_signin(&page, email, password, code.as_deref()).await?;
                    }
                    AuthType::Token(token) => {
                        handle_token(&page, token).await?;
                    }
                    AuthType::Cookies(_) => {
                        return Err(auth_error!("Invalid cookies"));
                    }
                }

                continue;
            }

            debug!("Already signed in");
            break;
        }
        let Some(results) = page
            .wait_for_selector_builder(".results-box")
            .state(FrameState::Visible)
            .wait_for_selector()
            .await?
        else {
            return Err(element_error!("Results box not found"));
        };
        let search_results = results.query_selector_all(".search-result").await?;
        let mut results = Vec::new();
        for result in &search_results {
            if results.len() >= limit {
                break;
            }
            let Some(title) = result.query_selector(".__sri-title").await? else {
                debug!("Title class not found");
                continue;
            };
            let title = title.inner_text().await?;
            let Some(url) = result.query_selector(".__sri-url-box").await? else {
                debug!("URL class not found");
                continue;
            };
            let Some(url) = url.query_selector("a").await? else {
                debug!("URL link not found");
                continue;
            };
            let Some(url) = url.get_attribute("href").await? else {
                debug!("URL attribute not found");
                continue;
            };
            let Some(snippet) = result.query_selector(".__sri-desc").await? else {
                debug!("Description class not found");
                continue;
            };
            let snippet = snippet.inner_text().await?;

            results.push(SearchResult {
                title,
                url,
                snippet,
            });
        }
        Ok(results)
    }
}
