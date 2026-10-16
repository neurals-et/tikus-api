use tikus_api::{Result, TikusClient};

#[tokio::main]
async fn main() -> Result<()> {
    println!("[*] Initializing Tikus Rust API client...");
    let client = TikusClient::new();

    println!("[*] Fetching public trending posts (popular tab)...");
    let feed = client.get_public_posts(Some("popular"), None, None).await?;
    println!("[+] Successfully loaded {} posts (has_more: {})", feed.posts.len(), feed.has_more);

    for (idx, post) in feed.posts.iter().take(5).enumerate() {
        let author = post.user.as_ref().map(|u| u.display_name()).unwrap_or_else(|| "Anonymous".to_string());
        let caption = post.caption.as_deref().unwrap_or("No caption").replace('\n', " ");
        println!("\n  [{}] {} by {}", idx + 1, post.id, author);
        println!("      Caption: \"{}\"", caption);
        println!("      Stats:   {} flames | {} comments | {} views", post.flames_count, post.comments_count, post.views_count);

        for media in &post.media {
            if let Some(url) = media.resolved_url() {
                println!("      -> {}: {}", media.media_type.to_uppercase(), url);
            }
        }
    }

    if let Some(cursor) = &feed.cursor {
        println!("\n[*] Fetching next page with cursor: {}...", cursor);
        let next_page = client.get_public_posts(Some("popular"), Some(cursor), None).await?;
        println!("[+] Loaded {} posts on page 2", next_page.posts.len());
    }

    Ok(())
}
