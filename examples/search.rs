use kagisearch::{AuthType, Browser};
use playwright::api::Cookie;
use tokio::io::AsyncBufReadExt;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, fmt};

const COOKIE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/target/cookies.json");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let auth_type = if tokio::fs::try_exists(COOKIE_PATH).await? {
        let content = tokio::fs::read(COOKIE_PATH).await?;
        let cookies: Vec<Cookie> = serde_json::from_str(&String::from_utf8_lossy(&content))?;
        AuthType::Cookies(cookies)
    } else {
        let mut reader = tokio::io::BufReader::new(tokio::io::stdin());

        println!("Which way do you want to sign in? (token/email)");
        let mut method = String::new();
        reader.read_line(&mut method).await?;
        match method.trim() {
            "token" => {
                let mut token = String::new();
                println!("Please input your Token:");
                reader.read_line(&mut token).await?;
                AuthType::Token(token.trim().to_string())
            }
            "email" => {
                let mut email = String::new();
                println!("Please input your Email or Username:");
                reader.read_line(&mut email).await?;
                let mut password = String::new();
                println!("Please input your Password:");
                reader.read_line(&mut password).await?;
                let mut code = String::new();
                println!("Please input your Two-factor Authentication code (if enabled):");
                reader.read_line(&mut code).await?;
                let code = code.trim();
                AuthType::Login(
                    email.trim().to_string(),
                    password.trim().to_string(),
                    if code.is_empty() {
                        None
                    } else {
                        Some(code.to_string())
                    },
                )
            }
            _ => {
                println!("Invalid method");
                return Ok(());
            }
        }
    };

    let save = !matches!(auth_type, AuthType::Cookies(_));

    let browser = Browser::new(auth_type).await?;
    let result = browser.search("What is Kagi Search", 5).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);

    if save {
        if let Some(cookies) = browser.cookies().await? {
            tokio::fs::write(COOKIE_PATH, serde_json::to_string(&cookies)?).await?;
        }
    }

    Ok(())
}
