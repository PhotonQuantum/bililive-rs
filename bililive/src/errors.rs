//! Error types.
use thiserror::Error;

use bililive_core::errors::Build as BuildError;
use bililive_core::errors::Parse;

/// The result type.
pub type Result<T> = std::result::Result<T, BililiveError>;

#[derive(Debug, Error)]
pub enum Stream {
    #[error("parse error: {0}")]
    Parse(#[from] Parse),
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
    #[error("ws error: {0}")]
    WebSocket(#[from] async_tungstenite::tungstenite::Error),
}

/// A wrapper type for `reqwest::Error`(tokio) or `http_client::Error`(async-std).
///
/// When `tokio-*` feature is enabled, HTTP requests are supported via `reqwest` crate.
///
/// When `async-*` feature is enabled, HTTP requests are supported via `http_client` crate.
///
/// Both crates have different error types. To make the error handling easier, a wrapper typed is
/// defined.
/// The main error type.
#[derive(Debug, Error)]
pub enum BililiveError {
    #[error("build error: {0}")]
    Build(#[from] BuildError),
    #[error("stream error: {0}")]
    Stream(#[from] Stream),
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("websocket error: {0}")]
    WebSocket(#[from] async_tungstenite::tungstenite::Error),
    #[error("client not connected")]
    NotConnected,
}
