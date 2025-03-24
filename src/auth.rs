use chromiumoxide::Page;
use url::Url;

use crate::{Error, auth_error, browser_error, consts::HOST};

/// Handle token authentication
pub(crate) async fn handle_token(page: &Page, token: &str) -> Result<(), Error> {
    let url = Url::parse_with_params(&format!("{}/search", HOST), &[("token", token)])?;
    page.goto(url).await?.wait_for_navigation().await?;
    let Some(url) = page.url().await? else {
        return Err(browser_error!("Failed to get URL"));
    };
    // Verify token
    if Url::parse(&url)?.path() != "/" {
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
    let form = page.find_element("#signInForm").await?;

    // Handle email input
    let input = form.find_element("input[name='email']").await?;
    input.click().await?.type_str(email).await?;

    // Handle password input
    let input = form.find_element("input[name='password']").await?;
    input.click().await?.type_str(password).await?;

    // submit the form
    form.find_element("button[type='submit']")
        .await?
        .click()
        .await?;
    page.wait_for_navigation().await?;

    let Some(url) = page.url().await? else {
        return Err(browser_error!("Failed to get URL"));
    };

    let mut url = Url::parse(&url)?;
    // Handle 2FA
    if url.path() == "/signin" {
        let Some(code) = code else {
            return Err(auth_error!("2FA code required"));
        };
        let form = page.find_element("#signInForm").await?;
        let input = form.find_element("input[name='code']").await?;
        input.click().await?.type_str(code).await?;
        form.find_element("button[type='submit']")
            .await?
            .click()
            .await?;
        page.wait_for_navigation().await?;

        let Some(new_url) = page.url().await? else {
            return Err(browser_error!("Failed to get URL"));
        };
        url = Url::parse(&new_url)?;
    }

    if url.path() != "/search" {
        return Err(auth_error!("Login failed"));
    }

    Ok(())
}
