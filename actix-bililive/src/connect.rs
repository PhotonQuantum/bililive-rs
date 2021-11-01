use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;

use actix_codec::Framed;
use awc::error::WsClientError;
use awc::http::{Uri, Version};
use awc::{BoxedSocket, Client};
use stream_reconnect::{ReconnectStream, UnderlyingStream};

use crate::core::config::StreamConfig;
use crate::core::errors::StreamError;
use crate::core::packet::Packet;
use crate::core::retry::{RetryConfig, RetryContext, WsStream, WsStreamTrait};
use crate::core::stream::HeartbeatStream;
use crate::stream::{Codec, PingPongStream};

/// Raw websocket stream type.
pub type InnerStream = PingPongStream<Framed<BoxedSocket, Codec>>;
/// Bililive stream type.
pub type DefaultStream = HeartbeatStream<InnerStream, WsClientError>;
/// Bililive stream type with auto-reconnect mechanism.
pub type RetryStream = ReconnectStream<
    WsStream<Connector, WsClientError>,
    RetryContext,
    Result<Packet, StreamError<WsClientError>>,
    StreamError<WsClientError>,
>;

pub struct Connector;

impl WsStreamTrait<WsClientError> for Connector {
    type Stream = DefaultStream;
    fn connect(
        url: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Stream, WsClientError>> + '_>> {
        let client = Client::builder()
            .max_http_version(Version::HTTP_11)
            .finish();
        let url = Uri::from_str(url).unwrap();
        Box::pin(async move {
            let (_, ws) = client.ws(url).connect().await?;
            let codec = ws.into_map_codec(Codec::new);
            Ok(HeartbeatStream::new(PingPongStream::new(codec)))
        })
    }
}

/// Connect to bilibili live room.
///
/// # Errors
/// Returns an error when websocket connection fails.
pub async fn connect(config: StreamConfig) -> Result<DefaultStream, StreamError<WsClientError>> {
    WsStream::<Connector, WsClientError>::establish(config.into()).await
}

/// Connect to bilibili live room with auto retry.
///
/// # Errors
/// Returns an error when websocket connection fails.
pub async fn connect_with_retry(
    stream_config: StreamConfig,
    retry_config: RetryConfig,
) -> Result<RetryStream, StreamError<WsClientError>> {
    let inner: RetryStream =
        ReconnectStream::connect_with_options(stream_config.into(), retry_config.into()).await?;
    Ok(inner)
}
