use std::error::Error as StdError;
use std::fmt::{Display, Formatter};

use nom::Needed;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, BililiveError>;

pub enum IncompleteResult<T> {
    Ok(T),
    Incomplete(Needed),
    Err(BililiveError),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("json error: {0}")]
    JSON(#[from] serde_json::Error),
    #[error("not a valid int32 big endian")]
    Int32BE,
    #[error("error when parsing room id")]
    RoomId,
    #[error("unknown websocket pack protocol")]
    UnknownProtocol,
    #[error("error when parsing packet struct")]
    PacketError(String),
    #[error("error when decompressing packet buffer: {0}")]
    ZlibError(#[from] std::io::Error),
}

/// A wrapper type for `http_client::Error`.
///
/// For some reason, `http_client::Error` doesn't implement Error trait.
/// To make it fit into BililiveError, we need to derive Error for it.
#[derive(Debug)]
pub struct HTTPError(http_client::Error);

impl From<http_client::Error> for HTTPError {
    fn from(e: http_client::Error) -> Self {
        Self(e)
    }
}

#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for HTTPError {
    fn from(e: reqwest::Error) -> Self {
        let status = e.status().map_or(500, |code| code.as_u16());
        Self(http_client::Error::new(status, e))
    }
}

impl StdError for HTTPError {}

impl Display for HTTPError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Error)]
pub enum BililiveError {
    #[error("http error: {0}")]
    HTTP(#[from] HTTPError),
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("build error: missing field {0}")]
    Build(String),
    #[error("websocket error: {0}")]
    WebSocket(#[from] async_tungstenite::tungstenite::Error),
    #[error("client not connected")]
    NotConnected,
}

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
