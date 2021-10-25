//! Error types.
use std::fmt::{Debug};

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
    RoomId, // TODO move this out (ref bililive::builder)
    #[error("unknown websocket pack protocol")]
    UnknownProtocol,
    #[error("error when parsing packet struct")]
    PacketError(String),
    #[error("error when decompressing packet buffer: {0}")]
    ZlibError(#[from] std::io::Error),
}