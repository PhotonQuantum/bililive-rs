use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, BililiveError>;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("json error: {0}")]
    JSON(#[from] serde_json::Error),
    #[error("error when parsing room id")]
    RoomId
}

#[derive(Debug, Error)]
pub enum BililiveError {
    #[error("http error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("build error: missing field {0}")]
    BuildError(String)
}