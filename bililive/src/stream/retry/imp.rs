macro_rules! impl_retry {
    ($adapter: ident) => {
        use std::future::Future;
        use std::io;
        use std::io::ErrorKind;
        use std::pin::Pin;

        use crate::core::packet::Packet;
        use async_tungstenite::tungstenite::error::{Error as WsError, Error};
        use async_tungstenite::tungstenite::Message;
        use async_tungstenite::$adapter::{connect_async, ConnectStream};
        use async_tungstenite::WebSocketStream;
        use futures::SinkExt;
        use stream_reconnect::UnderlyingStream;

        use super::RetryContext;

        impl UnderlyingStream<RetryContext, Result<Message, WsError>, WsError>
            for WebSocketStream<ConnectStream>
        {
            fn establish(
                mut ctor_arg: RetryContext,
            ) -> Pin<Box<dyn Future<Output = Result<Self, WsError>> + Send>> {
                Box::pin(async move {
                    let server = ctor_arg.get();
                    let (mut ws, _) = connect_async(server).await?;
                    ws.send(Message::binary(
                        Packet::new_room_enter(&ctor_arg.config).encode(),
                    ))
                    .await?;
                    Ok(ws)
                })
            }

            fn is_write_disconnect_error(&self, err: &WsError) -> bool {
                matches!(
                    err,
                    Error::ConnectionClosed
                        | Error::AlreadyClosed
                        | Error::Io(_)
                        | Error::Tls(_)
                        | Error::Protocol(_)
                )
            }

            fn is_read_disconnect_error(&self, item: &Result<Message, WsError>) -> bool {
                if let Err(e) = item {
                    self.is_write_disconnect_error(e)
                } else {
                    false
                }
            }

            fn exhaust_err() -> WsError {
                WsError::Io(io::Error::new(
                    ErrorKind::NotConnected,
                    "Disconnected. Connection attempts have been exhausted.",
                ))
            }
        }
    };
}
