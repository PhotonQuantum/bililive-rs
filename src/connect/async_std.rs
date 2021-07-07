use async_tungstenite::async_std::ConnectStream;
use async_tungstenite::tungstenite::{error::Error as WsError, Message};
use async_tungstenite::WebSocketStream;
use stream_reconnect::{ReconnectStream, UnderlyingStream};

use crate::config::{RetryConfig, StreamConfig};
use crate::errors::Result;
use crate::stream::retry::RetryContext;
use crate::stream::BililiveStream;

pub type InnerStream = WebSocketStream<ConnectStream>;
pub type InnerRetryStream = ReconnectStream<
    WebSocketStream<ConnectStream>,
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
