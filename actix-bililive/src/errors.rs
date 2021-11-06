//! Error types.
use awc::error::WsClientError;

pub use crate::core::errors::{BuildError, IncompleteResult, ParseError};

/// Errors that may occur when consuming a stream.
pub type StreamError = crate::core::errors::StreamError<WsClientError>;
