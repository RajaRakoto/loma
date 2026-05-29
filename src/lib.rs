//! loma — Local LLM Optimizer & Manager Assistant.
//!
//! # Modules
//! - [`api`]    — Axum HTTP server (routes, handlers, types)
//! - [`cli`]    — Clap CLI interface
//! - [`config`] — App configuration (env vars + .env)
//! - [`error`]  — Shared error type and Result alias

#![allow(non_snake_case)]

pub mod api;
pub mod cli;
pub mod commands;
pub mod config;
pub mod error;
pub mod utils;

pub use error::{Error, Result};

/// Application version (from Cargo.toml)
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name (from Cargo.toml)
pub const NAME: &str = env!("CARGO_PKG_NAME");
