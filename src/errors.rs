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

#[derive(Debug, Error)]
pub enum BililiveError {
    #[error("http error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("build error: missing field {0}")]
    Build(String),
    #[error("websocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("client not connected")]
    NotConnected,
    #[error("tokio join error")]
    JoinError(#[from] tokio::task::JoinError),
}
