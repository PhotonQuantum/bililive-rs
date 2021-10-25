//! Error types.
use std::fmt::{Debug, Display, Formatter};

use thiserror::Error;
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
    WebSocket(#[from] async_tungstenite::tungstenite::Error)
}

/// A wrapper type for `reqwest::Error`(tokio) or `http_client::Error`(async-std).
///
/// When `tokio-*` feature is enabled, HTTP requests are supported via `reqwest` crate.
///
/// When `async-*` feature is enabled, HTTP requests are supported via `http_client` crate.
///
/// Both crates have different error types. To make the error handling easier, a wrapper typed is
/// defined.
pub enum HTTPError {
    #[cfg(feature = "h1-client")]
    HTTPClient(http_client::Error),
    #[cfg(feature = "reqwest")]
    Reqwest(reqwest::Error),
}

#[cfg(feature = "h1-client")]
#[allow(unreachable_patterns)]
impl HTTPError {
    /// Get the inner error.
    #[must_use]
    pub fn inner(self) -> http_client::Error {
        match self {
            HTTPError::HTTPClient(e) => e,
            _ => unreachable!(),
        }
    }

    /// Get a reference to the inner error.
    #[must_use]
    pub fn inner_ref(&self) -> &http_client::Error {
        match self {
            HTTPError::HTTPClient(e) => e,
            _ => unreachable!(),
        }
    }
}

#[cfg(all(not(feature = "h1-client"), feature = "reqwest"))]
#[allow(unreachable_patterns)]
impl HTTPError {
    /// Get the inner error.
    #[must_use]
    pub fn inner(self) -> reqwest::Error {
        match self {
            HTTPError::Reqwest(e) => e,
            _ => unreachable!(),
        }
    }

    /// Get a reference to the inner error.
    #[must_use]
    pub fn inner_ref(&self) -> &reqwest::Error {
        match self {
            HTTPError::Reqwest(e) => e,
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "h1-client")]
impl From<http_client::Error> for HTTPError {
    fn from(e: http_client::Error) -> Self {
        Self::HTTPClient(e)
    }
}

#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for HTTPError {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl Debug for HTTPError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.inner_ref(), f)
    }
}

impl Display for HTTPError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.inner_ref(), f)
    }
}

/// The main error type.
#[derive(Debug, Error)]
pub enum BililiveError {
    #[error("http error: {0}")]
    HTTP(HTTPError),
    #[error("stream error: {0}")]
    Stream(#[from] Stream),
    #[error("parse: {0}")]
    Parse(#[from] Parse),   // TODO remove this (ref at builder)
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("build error: missing field {0}")]
    Build(String),
    #[error("websocket error: {0}")]
    WebSocket(#[from] async_tungstenite::tungstenite::Error),
    #[error("client not connected")]
    NotConnected,
}

#[cfg(feature = "h1-client")]
impl From<http_client::Error> for BililiveError {
    fn from(e: http_client::Error) -> Self {
        Self::HTTP(e.into())
    }
}

#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for BililiveError {
    fn from(e: reqwest::Error) -> Self {
        Self::HTTP(e.into())
    }
}
