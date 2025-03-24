# Kagi Search 🔍

[![Behavior Test](https://github.com/the-alchemists-of-arland/kagisearch/actions/workflows/test_behavior.yaml/badge.svg?branch=main)](https://github.com/the-alchemists-of-arland/kagisearch/blob/main/.github/workflows/test_behavior.yaml)
[![Crates.io](https://img.shields.io/crates/v/kagisearch.svg)](https://crates.io/crates/kagisearch)
[![Documentation](https://docs.rs/kagisearch/badge.svg)](https://docs.rs/kagisearch)
[![License](https://img.shields.io/badge/license-Apache-blue.svg)](LICENSE)

A Rust library that allows you to perform Kagi searches programmatically using chromiumoxide, without consuming additional API credits.

## ✨ Features

- 💳 No additional API credits required
- 🔐 Supports token-based, F2A-based and cookie-based authentication

## 📦 Installation

Add `kagisearch` to your `Cargo.toml`:

```toml
[dependencies]
kagisearch = "0.2.0"
```

## 🚀 Quick Start

```rust
use kagisearch::{AuthType, Kagi};

struct TokioSpawner;

impl Spawner for TokioSpawner {
    fn spawn(future: impl std::future::Future<Output = ()> + Send + 'static) {
        tokio::spawn(future);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the browser with your Kagi token
    let mut kagi = Kagi::new::<TokioSpawner>(AuthType::Token("your_token_here".to_string())).await?;
    // Perform a search and get up to 5 results
    let results = kagi.search("Rust programming language", 5).await?;

    let Some(results) = results else {
        return Err(anyhow::anyhow!("No search results found"));
    };

    // Print the results
    for result in results {
        println!("{:?}", result);
    }

    kagi.close().await?;

    Ok(())
}
```

## 📖 Documentation

For more detailed examples and usage instructions, check out:
- [Examples directory](./examples)
- [API Documentation](https://docs.rs/kagisearch)

## 🤝 Contributing

Contributions are welcome! Feel free to:
1. Fork the repository
2. Create a new branch for your feature
3. Submit a Pull Request

Please make sure to update tests as appropriate.

## Credits

- [chromiumoxide](https://github.com/mattsse/chromiumoxide)
- [google-search](https://github.com/web-agent-master/google-search)

## ⚖️ License

This project is licensed under the [Apache License](LICENSE).

## 📝 Note

> While Kagi is an excellent search engine, their API pricing can be cost-prohibitive. This library provides a way to integrate Kagi search functionality into your applications without incurring additional API costs beyond your Professional subscription.

