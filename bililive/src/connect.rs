//! Connection related functions and types.
macro_rules! impl_connect_mod {
    ($adapter:ident) => {
        use async_tungstenite::tungstenite::{error::Error as WsError, Message};
        use async_tungstenite::$adapter::ConnectStream;
        use async_tungstenite::WebSocketStream;
        use bililive_core::config::Stream as StreamConfig;
        use bililive_core::stream::HeartbeatStream;
        use stream_reconnect::{ReconnectStream, UnderlyingStream};

        use crate::config::RetryConfig;
        use crate::errors::Result;
        use crate::stream::retry::RetryContext;
        use crate::stream::CodecStream;

        pub type InnerStream = WebSocketStream<ConnectStream>;
        pub type InnerRetryStream = ReconnectStream<
            WebSocketStream<ConnectStream>,
            RetryContext,
            std::result::Result<Message, WsError>,
            WsError,
        >;
        pub type DefaultStream = HeartbeatStream<CodecStream<InnerStream>, WsError>;
        pub type RetryStream = HeartbeatStream<CodecStream<InnerRetryStream>, WsError>;

        /// Connect to bilibili live room.
        ///
        /// # Errors
        /// Returns an error when websocket connection fails.
        pub async fn connect(config: StreamConfig) -> Result<DefaultStream> {
            let inner = InnerStream::establish(config.into()).await?;
            Ok(HeartbeatStream::new(CodecStream::new(inner)))
        }

        /// Connect to bilibili live room with auto retry.
        ///
        /// # Errors
        /// Returns an error when websocket connection fails.
        pub async fn connect_with_retry(
            stream_config: StreamConfig,
            retry_config: RetryConfig,
        ) -> Result<RetryStream> {
            let inner: InnerRetryStream =
                ReconnectStream::connect_with_options(stream_config.into(), retry_config.into())
                    .await?;
            Ok(HeartbeatStream::new(CodecStream::new(inner)))
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
