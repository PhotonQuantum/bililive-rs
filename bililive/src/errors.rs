//! Error types.
use async_tungstenite::tungstenite::Error as WsError;

pub use crate::core::errors::{BuildError, IncompleteResult, ParseError};

/// Errors that may occur when consuming a stream.
pub type StreamError = crate::core::errors::StreamError<WsError>;
