use kagisearch::{AuthType, Kagi, Spawner};

struct TokioSpawner;

impl Spawner for TokioSpawner {
    fn spawn(future: impl std::future::Future<Output = ()> + Send + 'static) {
        tokio::spawn(future);
    }
}

#[tokio::test]
async fn test_search() -> anyhow::Result<()> {
    let token = std::env::var("KAGI_TOKEN")?;
    let mut kagi = Kagi::new::<TokioSpawner>(AuthType::Token(token)).await?;
    let results = kagi.search("Rust programming language", 5).await?;
    let Some(results) = results else {
        return Err(anyhow::anyhow!("No search results found"));
    };

    assert_eq!(results.len(), 5);
    for result in results {
        assert!(!result.title.is_empty());
        assert!(!result.url.is_empty());
        assert!(!result.snippet.is_empty());
    }
    kagi.close().await?;

    Ok(())
}
