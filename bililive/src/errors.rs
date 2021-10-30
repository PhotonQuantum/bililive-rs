//! Error types.
use async_tungstenite::tungstenite::Error as WsError;
use thiserror::Error;

use crate::core::errors::{Build as BuildError, Stream};

/// The result type.
pub type Result<T> = std::result::Result<T, BililiveError>;

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
    Stream(#[from] Stream<WsError>),
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("websocket error: {0}")]
    WebSocket(#[from] WsError),
    #[error("client not connected")]
    NotConnected,
}
