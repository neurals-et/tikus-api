//! Fast, asynchronous Rust API client for the **Tikus** Ethiopian social, short-video & live streaming platform (`com.tikus.tikus`).
//!
//! # Features
//! - **Public Feed & Video Exploration**: Fetch public feeds with pagination cursors, media metadata (CDN video/image signed URLs), engagement stats.
//! - **Authentication & Registration**: Send and verify SMS OTPs (`/send_otp`, `/verify_otp`), complete user account onboarding (`/register`), or authenticate with Google Native OAuth tokens.
//! - **Protected Feeds & Streaming**: Personalized feed (`/posts/feed`), active live stream channels (`/streams/live`), search suggestions, and virtual coin economy (`/coins`).
//! - **High Performance**: Pure async using `tokio` and `reqwest` with connection pooling and proxy support.

pub mod client;
pub mod error;
pub mod models;

pub use client::{TikusClient, TikusClientBuilder, DEFAULT_BASE_URL, DEFAULT_USER_AGENT};
pub use error::{Result, TikusError};
pub use models::{
    AuthResponse, LiveStream, MediaItem, MediaResource, OtpResponse, PaginatedPosts, TikusPost,
    TikusUser,
};
