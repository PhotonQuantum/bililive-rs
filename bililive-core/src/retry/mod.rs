use std::future::Future;
use std::io;
use std::io::ErrorKind;
use std::marker::PhantomData;
use std::pin::Pin;

use async_trait::async_trait;
use futures::SinkExt;
use futures::{Sink, Stream};
use stream_reconnect::UnderlyingStream;

use context::RetryContext;

use crate::errors::Stream as StreamError;
use crate::packet::Packet;

pub mod config;
pub mod context;
pub mod policy;

#[cfg(feature = "not-send")]
#[async_trait(?Send)]
pub trait WsStreamTrait<E> {
    type Stream: Stream<Item = Result<Packet, StreamError<E>>>
        + Sink<Packet, Error = StreamError<E>>
        + Unpin
        + Sized;
    async fn connect(url: &str) -> Result<Self::Stream, E>;
}

#[cfg(not(feature = "not-send"))]
#[async_trait]
pub trait WsStreamTrait<E> {
    type Stream: Stream<Item = Result<Packet, StreamError<E>>>
    + Sink<Packet, Error = StreamError<E>>
    + Unpin
    + Sized
    + Send;
    async fn connect(url: &str) -> Result<Self::Stream, E>;
}

#[derive(Debug, Default)]
pub struct WsStream<T: WsStreamTrait<E>, E>(PhantomData<(T, E)>);

impl<T, E> WsStream<T, E>
where
    T: WsStreamTrait<E>,
{
    /// # Errors
    /// Returns an error when websocket connection fails.
    pub async fn connect(url: &str) -> Result<T::Stream, E> {
        T::connect(url).await
    }
}

#[allow(clippy::type_complexity)]
impl<T, E> UnderlyingStream<RetryContext, Result<Packet, StreamError<E>>, StreamError<E>>
    for WsStream<T, E>
where
    T: WsStreamTrait<E>,
    E: std::error::Error,
{
    type Stream = T::Stream;

    #[cfg(feature = "not-send")]
    fn establish(
        mut ctor_arg: RetryContext,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Stream, StreamError<E>>>>> {
        Box::pin(async move {
            let server = ctor_arg.get();
            let mut ws = Self::connect(server)
                .await
                .map_err(StreamError::from_ws_error)?;
            ws.send(Packet::new_room_enter(ctor_arg.config())).await?;
            Ok(ws)
        })
    }

    #[cfg(not(feature = "not-send"))]
    fn establish(
        mut ctor_arg: RetryContext,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Stream, StreamError<E>>> + Send>> {
        Box::pin(async move {
            let server = ctor_arg.get();
            let mut ws = Self::connect(server)
                .await
                .map_err(StreamError::from_ws_error)?;
            ws.send(Packet::new_room_enter(ctor_arg.config())).await?;
            Ok(ws)
        })
    }

    fn is_write_disconnect_error(err: &StreamError<E>) -> bool {
        matches!(err, StreamError::WebSocket(_) | StreamError::IO(_))
    }

    fn is_read_disconnect_error(item: &Result<Packet, StreamError<E>>) -> bool {
        if let Err(e) = item {
            Self::is_write_disconnect_error(e)
        } else {
            false
        }
    }

    fn exhaust_err() -> StreamError<E> {
        StreamError::IO(io::Error::new(
            ErrorKind::NotConnected,
            "Disconnected. Connection attempts have been exhausted.",
        ))
    }
}
