//! Traits and types used by retry mechanism.

use std::future::Future;
use std::io;
use std::io::ErrorKind;
use std::marker::PhantomData;
use std::pin::Pin;

use futures::SinkExt;
use futures::{Sink, Stream};
use stream_reconnect::UnderlyingStream;

pub use config::RetryConfig;
pub use context::RetryContext;
pub use policy::BEBIterator;

use crate::errors::StreamError;
use crate::packet::Packet;

mod config;
mod context;
mod policy;

/// Trait of helper objects to connect bilibili websocket server.
///
/// This trait is used when constructing normal bililive streams or auto-retry bililive streams.
///
/// An implementation of `WsStreamTrait` takes in a ws server url and decodes the data into a stream
/// of [`Packet`](crate::packet::Packet) with heartbeat auto-response mechanism implemented
/// (see [`HeartbeatStream`](crate::stream::HeartbeatStream) for details).
#[cfg(feature = "not-send")]
pub trait WsStreamTrait<E> {
    /// The returned stream type.
    type Stream: Stream<Item = Result<Packet, StreamError<E>>>
        + Sink<Packet, Error = StreamError<E>>
        + Unpin
        + Sized;
    /// Connect to bilibili websocket server.
    ///
    /// # Errors
    /// Returns an error when websocket connection fails.
    fn connect(url: &str) -> Pin<Box<dyn Future<Output = Result<Self::Stream, E>> + '_>>;
}

#[cfg(not(feature = "not-send"))]
pub trait WsStreamTrait<E> {
    /// The returned stream type.
    type Stream: Stream<Item = Result<Packet, StreamError<E>>>
        + Sink<Packet, Error = StreamError<E>>
        + Unpin
        + Sized
        + Send;
    /// Connect to bilibili websocket server.
    ///
    /// # Errors
    /// Returns an error when websocket connection fails.
    fn connect(url: &str) -> Pin<Box<dyn Future<Output = Result<Self::Stream, E>> + Send + '_>>;
}

/// Wrapper for types implementing `WsStreamTrait`.
///
/// This type is used to avoid the orphan rule. Exposed for stream type construction.
#[derive(Debug, Default)]
pub struct WsStream<T: WsStreamTrait<E>, E>(PhantomData<(T, E)>);

impl<T, E> WsStream<T, E>
where
    T: WsStreamTrait<E>,
{
    /// Connect to bilibili websocket server.
    ///
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
