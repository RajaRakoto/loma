//! Shared error types for rust-boilerplate.
//!
//! Uses [`thiserror`] for ergonomic error definitions.
//! All modules return [`Result<T>`] which aliases `std::result::Result<T, Error>`.

use thiserror::Error;

/// Application-wide error type.
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration or environment variable errors.
    #[error("Config error: {0}")]
    Config(String),

    /// I/O errors (file, socket, …).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialisation / deserialisation errors.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP API server errors.
    #[error("API error: {0}")]
    Api(String),

    /// Input validation errors.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Catch-all for unexpected errors.
    #[error("{0}")]
    Other(String),
}

/// Convenience alias — every module uses `crate::Result<T>`.
pub type Result<T> = std::result::Result<T, Error>;

// ── Constructor helpers ────────────────────────────────────────────────────────

impl Error {
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }
    pub fn api(msg: impl Into<String>) -> Self {
        Self::Api(msg.into())
    }
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Self::Other(s)
    }
}
impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::Other(s.to_owned())
    }
}
