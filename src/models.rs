use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AvatarImage {
    #[serde(rename = "signedPathUrl")]
    pub signed_path_url: Option<String>,
    pub path: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AvatarWrapper {
    pub image: Option<AvatarImage>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StreakInfo {
    pub current_streak: Option<i64>,
    pub longest_streak: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TikusUser {
    #[serde(default)]
    pub id: i64,
    pub tikus_user_id: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: Option<String>,
    pub gender: Option<String>,
    pub avatar: Option<AvatarWrapper>,
    #[serde(default)]
    pub is_under_age: bool,
    #[serde(default)]
    pub is_live: bool,
    pub live_stream_id: Option<String>,
    pub streak: Option<StreakInfo>,
}

impl TikusUser {
    /// Convenience helper to retrieve the resolved avatar URL.
    pub fn avatar_url(&self) -> Option<&str> {
        self.avatar
            .as_ref()
            .and_then(|a| a.image.as_ref())
            .and_then(|img| img.signed_path_url.as_deref().or(img.path.as_deref()))
    }

    /// Full display name.
    pub fn display_name(&self) -> String {
        match (&self.first_name, &self.last_name) {
            (Some(f), Some(l)) => format!("{} {}", f, l),
            (Some(f), None) => f.clone(),
            (None, Some(l)) => l.clone(),
            (None, None) => self.username.clone().unwrap_or_else(|| "Anonymous".to_string()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MediaResource {
    #[serde(rename = "signedPathUrl")]
    pub signed_path_url: Option<String>,
    pub path: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MediaItem {
    #[serde(default)]
    pub id: i64,
    #[serde(default = "default_media_type")]
    pub media_type: String, // "image" or "video"
    pub preview_path: Option<String>,
    pub video: Option<MediaResource>,
    pub image: Option<MediaResource>,
}

fn default_media_type() -> String {
    "image".to_string()
}

impl MediaItem {
    /// Resolved direct media CDN URL.
    pub fn resolved_url(&self) -> Option<&str> {
        if self.media_type == "video" {
            self.video
                .as_ref()
                .and_then(|v| v.signed_path_url.as_deref().or(v.path.as_deref()))
        } else {
            self.image
                .as_ref()
                .and_then(|i| i.signed_path_url.as_deref().or(i.path.as_deref()))
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TikusPost {
    #[serde(default)]
    pub id: i64,
    pub tikus_post_id: Option<String>,
    pub user_id: Option<i64>,
    pub caption: Option<String>,
    pub tag: Option<String>,
    #[serde(default = "default_post_status")]
    pub status: String,
    pub created_at: Option<String>,
    #[serde(default)]
    pub flames_count: i64,
    #[serde(default)]
    pub comments_count: i64,
    #[serde(default)]
    pub views_count: i64,
    #[serde(default)]
    pub shares_count: i64,
    pub user: Option<TikusUser>,
    #[serde(default)]
    pub media: Vec<MediaItem>,
}

fn default_post_status() -> String {
    "active".to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PaginatedPosts {
    #[serde(default)]
    pub posts: Vec<TikusPost>,
    pub cursor: Option<String>,
    #[serde(default)]
    pub has_more: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct OtpResponse {
    pub status: Option<i64>,
    pub message: Option<String>,
    pub code: Option<String>,
    pub registration_token: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AuthResponse {
    pub status: Option<i64>,
    pub token: Option<String>,
    pub access_token: Option<String>,
    pub registration_token: Option<String>,
    pub user: Option<TikusUser>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LiveStream {
    pub id: Option<String>,
    pub channel_id: Option<String>,
    pub host_id: Option<String>,
    pub title: Option<String>,
    pub viewers_count: Option<i64>,
    pub stream_url: Option<String>,
}
