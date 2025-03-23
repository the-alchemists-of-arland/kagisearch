use kagisearch::{AuthType, Browser};

#[tokio::test]
async fn test_search() -> anyhow::Result<()> {
    let token = std::env::var("KAGI_TOKEN")?;
    let browser = Browser::new(AuthType::Token(token)).await?;
    let results = browser.search("Rust programming language", 5).await?;

    assert_eq!(results.len(), 5);
    for result in results {
        assert!(!result.title.is_empty());
        assert!(!result.url.is_empty());
        assert!(!result.snippet.is_empty());
    }

    Ok(())
}
