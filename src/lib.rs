use stream_reconnect::{ReconnectStream, UnderlyingStream};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub use builder::*;
pub use errors::*;
pub use packet::*;
pub use stream::BililiveStream;

pub use crate::config::*;
use crate::errors::Result;
use crate::stream::retry::RetryContext;

#[macro_use]
mod utils;
mod builder;
mod config;
mod errors;
pub mod packet;
pub mod stream;

type InnerStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type InnerRetryStream = ReconnectStream<
    WebSocketStream<MaybeTlsStream<TcpStream>>,
    RetryContext,
    std::result::Result<Message, WsError>,
    WsError,
>;
pub type DefaultStream = BililiveStream<InnerStream>;
pub type RetryStream = BililiveStream<InnerRetryStream>;

pub async fn connect(config: StreamConfig) -> Result<DefaultStream> {
    let inner = InnerStream::establish(config.into()).await?;
    Ok(BililiveStream::new(inner))
}

pub async fn connect_with_retry(
    stream_config: StreamConfig,
    retry_config: RetryConfig,
) -> Result<RetryStream> {
    let inner: InnerRetryStream =
        ReconnectStream::connect_with_options(stream_config.into(), retry_config.into()).await?;
    Ok(BililiveStream::new(inner))
}
