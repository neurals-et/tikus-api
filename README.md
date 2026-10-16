# tikus-api (Rust)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Fast, strongly typed, asynchronous Rust API client for the **Tikus** Ethiopian social, short-video & live streaming platform (`com.tikus.tikus`).

---

## Features

- **High-Performance Async**: Built on top of `tokio` and `reqwest` with automatic HTTP connection pooling and optional proxy support.
- **Public Feed & Video Exploration**: Fetch public feeds with pagination cursors, media metadata (CDN video/image signed URLs), and engagement stats (flames, comments, views, shares).
- **Authentication & Registration**: Send and verify SMS OTPs (`/send_otp`, `/verify_otp`), complete user account onboarding (`/register`), or authenticate with Google Native OAuth tokens.
- **Protected Feeds & Streaming**: Personalized feed (`/posts/feed`), active live stream channels (`/streams/live`), search suggestions, and virtual coin economy (`/coins`).
- **Strongly Typed**: Clean serde-backed data structures for `TikusPost`, `TikusUser`, `MediaItem`, `PaginatedPosts`, `OtpResponse`, and `AuthResponse`.

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
tikus-api = { path = "path/to/tikus-api-rs" }
tokio = { version = "1", features = ["full"] }
```

---

## Quickstart

### 1. Fetch Public Trending Posts

```rust
use tikus_api::{TikusClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = TikusClient::new();

    let feed = client.get_public_posts(Some("popular"), None, None).await?;
    println!("Loaded {} posts (has_more: {})", feed.posts.len(), feed.has_more);

    for post in &feed.posts {
        let author = post.user.as_ref().map(|u| u.display_name()).unwrap_or_default();
        println!("[{}] by {} | Flames: {}", post.id, author, post.flames_count);
        for media in &post.media {
            if let Some(url) = media.resolved_url() {
                println!("  -> {}: {}", media.media_type, url);
            }
        }
    }

    Ok(())
}
```

### 2. Configure Client with Proxy and Token

```rust
use std::time::Duration;
use tikus_api::TikusClient;

let client = TikusClient::builder()
    .timeout(Duration::from_secs(30))
    .proxy("http://proxy.example.com:8080")
    .token("YOUR_BEARER_TOKEN")
    .build()?;
```

### 3. Send SMS OTP & Register Account

```rust
use tikus_api::{TikusClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = TikusClient::new();

    // 1. Request OTP
    client.send_otp("+251", "911223344").await?;
    println!("OTP sent!");

    // 2. Verify OTP
    let v_res = client.verify_otp("+251", "911223344", "123456").await?;

    // 3. Register user
    let auth = client.register(
        "Daniel",
        "male",
        "+251",
        "911223344",
        Some("123456"),
        v_res.registration_token.as_deref(),
        Some("Delaney"),
    ).await?;

    println!("Registered! Token: {:?}", auth.token);
    Ok(())
}
```

---

## Examples

Run the bundled examples:

```bash
cargo run --example explore
cargo run --example auth
```

---

## License

MIT License. See [LICENSE](LICENSE) for details.
