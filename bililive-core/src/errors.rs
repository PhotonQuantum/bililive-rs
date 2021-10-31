//! Error types.
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

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

/// Errors that may occur when making HTTP requests through builder.
#[derive(Debug)]
pub struct BuildError(pub(crate) Box<dyn std::error::Error>);

impl Display for BuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Error for BuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }

    #[allow(deprecated)]
    fn description(&self) -> &str {
        self.0.description()
    }

    #[allow(deprecated)]
    fn cause(&self) -> Option<&dyn Error> {
        self.0.cause()
    }
}

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
