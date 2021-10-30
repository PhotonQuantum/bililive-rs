use actix_codec::Framed;
use async_trait::async_trait;
use awc::{BoxedSocket, Client};
use awc::error::WsProtocolError;
use awc::http::Version;
use stream_reconnect::{ReconnectStream, UnderlyingStream};

use crate::core::config::Stream as StreamConfig;
use crate::core::errors::Stream as StreamError;
use crate::core::packet::Packet;
use crate::core::retry::{WsStream, WsStreamTrait};
use crate::core::retry::config::RetryConfig;
use crate::core::retry::context::RetryContext;
use crate::core::stream::HeartbeatStream;
use crate::stream::codec::Codec;
use crate::stream::pingpong::PingPong;

pub type InnerStream = PingPong<Framed<BoxedSocket, Codec>>;
pub type DefaultStream = HeartbeatStream<InnerStream, WsProtocolError>;
pub type RetryStream = ReconnectStream<
    WsStream<Connector, WsProtocolError>,
    RetryContext,
    std::result::Result<Packet, StreamError<WsProtocolError>>,
    StreamError<WsProtocolError>,
>;

pub struct Connector;

#[async_trait(? Send)]
impl WsStreamTrait<WsProtocolError> for Connector {
    type Stream = DefaultStream;
    async fn connect(url: &str) -> Result<Self::Stream, WsProtocolError> {
        let client = Client::builder().max_http_version(Version::HTTP_11).finish();
        let (_, ws) = client.ws(url).connect().await.unwrap();   // TODO error handling
        let codec = ws.into_map_codec(Codec::new);
        Ok(HeartbeatStream::new(PingPong::new(codec)))
    }
}

/// Connect to bilibili live room.
///
/// # Errors
/// Returns an error when websocket connection fails.
pub async fn connect(config: StreamConfig) -> Result<DefaultStream, StreamError<WsProtocolError>> {
    WsStream::<Connector, WsProtocolError>::establish(config.into()).await
}

/// Connect to bilibili live room with auto retry.
///
/// # Errors
/// Returns an error when websocket connection fails.
pub async fn connect_with_retry(
    stream_config: StreamConfig,
    retry_config: RetryConfig,
) -> Result<RetryStream, StreamError<WsProtocolError>> {
    let inner: RetryStream =
        ReconnectStream::connect_with_options(stream_config.into(), retry_config.into())
            .await?;
    Ok(inner)
}
