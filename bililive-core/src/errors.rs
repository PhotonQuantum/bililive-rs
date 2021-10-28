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
    Err(Parse),
}

/// Errors that may occur when parsing a packet.
#[derive(Debug, Error)]
pub enum Parse {
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not a valid int32 big endian")]
    Int32BE,
    #[error("error when parsing room id")]
    RoomId,
    // TODO move this out (ref bililive::builder)
    #[error("unknown websocket pack protocol")]
    UnknownProtocol,
    #[error("error when parsing packet struct")]
    PacketError(String),
    #[error("error when decompressing packet buffer: {0}")]
    ZlibError(#[from] std::io::Error),
}

// Errors that may occur when requesting through builder
#[derive(Debug)]
pub struct Build(pub(crate) Box<dyn std::error::Error + Send + Sync>);

impl Display for Build {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Error for Build {
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
