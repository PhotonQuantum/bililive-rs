use nom::Needed;
use thiserror::Error;

#[cfg(feature = "h1-client")]
use h1_wrapper::HTTPClientError;

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

#[cfg(feature = "h1-client")]
mod h1_wrapper {
    use std::error::Error as StdError;
    use std::fmt::{Display, Formatter};

    /// A wrapper type for `http_client::Error`.
    ///
    /// For some reason, `http_client::Error` doesn't implement Error trait.
    /// To make it fit into BililiveError, we need to derive Error for it.
    #[derive(Debug)]
    pub struct HTTPClientError(http_client::Error);

    impl From<http_client::Error> for HTTPClientError {
        fn from(e: http_client::Error) -> Self {
            Self(e)
        }
    }

    impl StdError for HTTPClientError {}

    impl Display for HTTPClientError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }
}

#[derive(Debug, Error)]
pub enum BililiveError {
    #[cfg(feature = "h1-client")]
    #[error("http error: {0}")]
    HTTP(HTTPClientError),
    #[cfg(feature = "reqwest")]
    #[error("http error: {0}")]
    HTTP(#[from] reqwest::Error),
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

#[cfg(feature = "h1-client")]
impl From<http_client::Error> for BililiveError {
    fn from(e: http_client::Error) -> Self {
        Self::HTTP(e.into())
    }
}
