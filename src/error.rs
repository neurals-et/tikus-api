use thiserror::Error;

#[derive(Error, Debug)]
pub enum TikusError {
    #[error("HTTP transport error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Authentication required or token expired: {0}")]
    Authentication(String),

    #[error("OTP error: {0}")]
    Otp(String),

    #[error("Validation failed (422): {0}")]
    Validation(String),

    #[error("Resource not found (404): {0}")]
    NotFound(String),

    #[error("Tikus API error (HTTP {status}): {message}")]
    Api { status: u16, message: String },
}

pub type Result<T> = std::result::Result<T, TikusError>;
