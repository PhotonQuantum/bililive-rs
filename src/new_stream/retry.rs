use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;

use futures::SinkExt;
use stream_reconnect::UnderlyingStream;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::error::{Error as WsError, Error};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::config::StreamConfig;

use super::utils::room_enter_message;
use std::io::ErrorKind;
use std::io;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Clone)]
pub struct RetryContext {
    config: StreamConfig,
    cursor: Arc<AtomicUsize>,
}

impl RetryContext {
    pub fn get(&mut self) -> &str {
        let cursor: usize = self
            .cursor
            .fetch_update(SeqCst, SeqCst, |i| Some((i + 1) % self.config.servers.len()))
            .unwrap();
        &*self.config.servers[cursor]
    }
}

impl From<StreamConfig> for RetryContext {
    fn from(config: StreamConfig) -> Self {
        Self {
            config,
            cursor: Arc::new(Default::default())
        }
    }
}

impl UnderlyingStream<RetryContext, Result<Message, WsError>, WsError> for WebSocketStream<MaybeTlsStream<TcpStream>> {
    fn establish(
        mut ctor_arg: RetryContext,
    ) -> Pin<Box<dyn Future<Output = Result<Self, WsError>> + Send>> {
        Box::pin(async move {
            let server = ctor_arg.get();
            let (mut ws, _) = connect_async(server).await?;
            ws.send(room_enter_message(&ctor_arg.config)).await?;
            Ok(ws)
        })
    }

    fn is_write_disconnect_error(&self, err: &WsError) -> bool {
        matches!(err, Error::ConnectionClosed
            | Error::AlreadyClosed
            | Error::Io(_)
            | Error::Tls(_)
            | Error::Protocol(_))
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