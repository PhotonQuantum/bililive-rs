//! Error types.
use std::fmt::Debug;

use nom::Needed;
use thiserror::Error;

/// The result returned by parsing functions.
///
/// * `Ok` indicates a successful parse.
/// * `Incomplete` means that more data is needed to complete the parsing.
/// The `Needed` enum can contain how many additional bytes are necessary.
/// * `Err` indicates an error.
pub enum IncompleteResult<T> {
    Ok(T),
    Incomplete(Needed),
    Err(ParseError),
}

/// Errors that may occur when parsing a packet.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not a valid int32 big endian")]
    Int32BE,
    #[error("unknown websocket pack protocol")]
    UnknownProtocol,
    #[error("error when parsing packet struct")]
    PacketError(String),
    #[error("error when decompressing packet buffer: {0}")]
    ZlibError(#[from] std::io::Error),
}

#[cfg(feature = "not-send")]
pub(crate) type BoxedError = Box<dyn std::error::Error>;

#[cfg(not(feature = "not-send"))]
pub(crate) type BoxedError = Box<dyn std::error::Error + Send + Sync>;

/// Errors that may occur when making HTTP requests through builder.
#[derive(Debug, Error)]
#[error("error when making http request: {0}")]
pub struct BuildError(#[source] pub(crate) BoxedError);

/// Errors that may occur when consuming a stream.
///
/// `E` is determined by the underlying websocket implementation.
#[derive(Debug, Error)]
pub enum StreamError<E> {
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("ws error: {0}")]
    WebSocket(E),
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
}

impl<E> StreamError<E> {
    pub const fn from_ws_error(e: E) -> Self {
        Self::WebSocket(e)
    }
}
