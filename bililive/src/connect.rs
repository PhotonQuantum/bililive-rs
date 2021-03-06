//! Connection related functions and types.
macro_rules! impl_connect_mod {
    ($adapter:ident) => {
        use std::future::Future;
        use std::pin::Pin;
        use std::str::FromStr;

        use async_tungstenite::tungstenite::error::Error as WsError;
        use async_tungstenite::$adapter::{connect_async, ConnectStream};
        use async_tungstenite::WebSocketStream;
        use stream_reconnect::{ReconnectStream, UnderlyingStream};
        use url::Url;

        use crate::core::config::StreamConfig;
        use crate::core::errors::StreamError;
        use crate::core::packet::Packet;
        use crate::core::retry::{RetryConfig, RetryContext, WsStream, WsStreamTrait};
        use crate::core::stream::HeartbeatStream;
        use crate::stream::CodecStream;

        /// Raw websocket stream type.
        pub type InnerStream = WebSocketStream<ConnectStream>;
        /// Bililive stream type.
        pub type DefaultStream = HeartbeatStream<CodecStream<InnerStream>, WsError>;
        /// Bililive stream type with auto-reconnect mechanism.
        pub type RetryStream = ReconnectStream<
            WsStream<Connector, WsError>,
            RetryContext,
            Result<Packet, StreamError<WsError>>,
            StreamError<WsError>,
        >;

        #[doc(hidden)]
        pub struct Connector;

        impl WsStreamTrait<WsError> for Connector {
            type Stream = DefaultStream;
            fn connect(
                url: &str,
            ) -> Pin<Box<dyn Future<Output = Result<Self::Stream, WsError>> + Send + '_>> {
                let url = Url::from_str(url).unwrap();
                Box::pin(async move {
                    Ok(HeartbeatStream::new(CodecStream::new(
                        connect_async(url).await?.0,
                    )))
                })
            }
        }

        /// Connect to bilibili live room.
        ///
        /// # Errors
        /// Returns an error when websocket connection fails.
        pub async fn connect(config: StreamConfig) -> Result<DefaultStream, StreamError<WsError>> {
            WsStream::<Connector, WsError>::establish(config.into()).await
        }

        /// Connect to bilibili live room with auto retry.
        ///
        /// # Errors
        /// Returns an error when websocket connection fails.
        pub async fn connect_with_retry(
            stream_config: StreamConfig,
            retry_config: RetryConfig,
        ) -> Result<RetryStream, StreamError<WsError>> {
            let inner: RetryStream =
                ReconnectStream::connect_with_options(stream_config.into(), retry_config.into())
                    .await?;
            Ok(inner)
        }
    };
}

#[cfg(feature = "tokio")]
pub mod tokio {
    //! `tokio` integration.
    impl_connect_mod!(tokio);
}

#[cfg(feature = "async-std")]
pub mod async_std {
    //! `async_std` integration.
    impl_connect_mod!(async_std);
}
