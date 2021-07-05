pub use builder::*;
pub use errors::*;
pub use new_stream::BililiveStreamNew;
pub use packet::*;
pub use stream::*;
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::tungstenite::{Error as WsError, Message};
use tokio::net::TcpStream;
use crate::errors::Result;
use stream_reconnect::{UnderlyingStream, ReconnectStream};
use crate::new_stream::retry::RetryContext;

#[macro_use]
mod utils;
mod builder;
mod config;
mod errors;
pub mod new_stream;
pub mod packet;
mod stream;

type InnerStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type InnerRetryStream = ReconnectStream<WebSocketStream<MaybeTlsStream<TcpStream>>, RetryContext, std::result::Result<Message, WsError>, WsError>;
pub type DefaultStream = BililiveStreamNew<InnerStream>;
pub type RetryStream = BililiveStreamNew<InnerRetryStream>;

pub async fn connect(config: &StreamConfig) -> Result<DefaultStream> {
    let inner = InnerStream::establish(config.clone().into()).await?;
    Ok(BililiveStreamNew::new(inner))
}

pub async fn connect_with_retry(config: &StreamConfig) -> Result<RetryStream> {
    let inner: InnerRetryStream = ReconnectStream::connect(config.clone().into()).await?;
    Ok(BililiveStreamNew::new(inner))
}
