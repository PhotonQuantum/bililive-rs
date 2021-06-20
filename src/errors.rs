use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, BililiveError>;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("json error: {0}")]
    JSON(#[from] serde_json::Error),
    #[error("error when parsing room id")]
    RoomId,
    #[error("unknown websocket pack protocol")]
    UnknownProtocol,
    #[error("error when parsing packet")]
    PacketError(String),
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
