use playwright::api::Page;
use url::Url;

use crate::{Error, auth_error, consts::HOST, element_error};

/// Handle token authentication
pub(crate) async fn handle_token(page: &Page, token: &str) -> Result<(), Error> {
    let url = Url::parse_with_params(&format!("{}/search", HOST), &[("token", token)])?;
    page.goto_builder(url.as_str()).goto().await?;
    // Verify token
    if Url::parse(&page.url()?)?.path() != "/" {
        return Err(auth_error!("Invalid token"));
    }
    Ok(())
}

/// Handle login authentication
pub(crate) async fn handle_signin(
    page: &Page,
    email: &str,
    password: &str,
    code: Option<&str>,
) -> Result<(), Error> {
    // Fill in the login form
    let form = page
        .wait_for_selector_builder("#signInForm")
        .wait_for_selector()
        .await?
        .ok_or_else(|| element_error!("Sign in form not found"))?;

    // Handle email input
    let input = form
        .query_selector("input[name='email']")
        .await?
        .ok_or_else(|| element_error!("Email input not found"))?;
    input.type_builder(email).r#type().await?;

    // Handle password input
    let input = form
        .query_selector("input[name='password']")
        .await?
        .ok_or_else(|| element_error!("Password input not found"))?;
    input.type_builder(password).r#type().await?;

    // Handle verification code
    if let Some(code) = code {
        let input = form
            .query_selector("input[name='code']")
            .await?
            .ok_or_else(|| element_error!("Code input not found"))?;
        input.type_builder(code).r#type().await?;
    }

    // Submit login form
    let button = form
        .query_selector("button[type='submit']")
        .await?
        .ok_or_else(|| element_error!("Submit button not found"))?;
    button.click_builder().click().await?;

    if Url::parse(&page.url()?)?.path() == "/search" {
        return Ok(());
    }

    Err(auth_error!("Login failed"))
}
