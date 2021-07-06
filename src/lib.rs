pub use builder::*;
pub use errors::*;
pub use stream::BililiveStream;
pub use packet::*;
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::tungstenite::{Error as WsError, Message};
use tokio::net::TcpStream;
use crate::errors::Result;
use stream_reconnect::{UnderlyingStream, ReconnectStream};
use crate::stream::retry::RetryContext;
use crate::config::StreamConfig;

#[macro_use]
mod utils;
mod builder;
mod config;
mod errors;
pub mod stream;
pub mod packet;

type InnerStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type InnerRetryStream = ReconnectStream<WebSocketStream<MaybeTlsStream<TcpStream>>, RetryContext, std::result::Result<Message, WsError>, WsError>;
pub type DefaultStream = BililiveStream<InnerStream>;
pub type RetryStream = BililiveStream<InnerRetryStream>;

pub async fn connect(config: StreamConfig) -> Result<DefaultStream> {
    let inner = InnerStream::establish(config.into()).await?;
    Ok(BililiveStream::new(inner))
}

pub async fn connect_with_retry(config: StreamConfig) -> Result<RetryStream> {
    let inner: InnerRetryStream = ReconnectStream::connect(config.into()).await?;
    Ok(BililiveStream::new(inner))
}
