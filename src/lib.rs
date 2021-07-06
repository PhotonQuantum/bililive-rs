use stream_reconnect::{ReconnectStream, UnderlyingStream};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::errors::Result;
use crate::stream::retry::RetryContext;
pub use crate::stream::BililiveStream;
pub use crate::builder::ConfigBuilder;
pub use crate::config::{StreamConfig, RetryConfig};
pub use crate::errors::BililiveError;
pub use crate::packet::{Packet, Protocol, Operation};

#[macro_use]
mod utils;
pub mod builder;
pub mod config;
pub mod errors;
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
