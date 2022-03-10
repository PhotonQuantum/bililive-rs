use std::pin::Pin;
use std::task::{Context, Poll};

use async_tungstenite::tungstenite::Error as WsError;
use async_tungstenite::tungstenite::Message;
use futures::ready;
use futures::{Sink, Stream};
use log::{debug, warn};

use crate::core::errors::IncompleteResult;
use crate::core::errors::StreamError;
use crate::core::packet::Packet;

/// A stream/sink interface to underlying websocket frame stream. Encodes/decodes bilibili live packets.
pub struct CodecStream<T> {
    /// underlying tungstenite stream
    stream: T,
    /// rx buffer
    read_buffer: Vec<u8>,
}

impl<T> CodecStream<T> {
    /// Convert a tungstenite stream into a [`CodecStream`](CodecStream).
    ///
    /// You may want to use `connect` or `connect_with_retry` in [`connect`](crate::connect) module instead.
    pub const fn new(stream: T) -> Self {
        Self {
            stream,
            read_buffer: vec![],
        }
    }
}

impl<T> Stream for CodecStream<T>
where
    T: Stream<Item = Result<Message, WsError>> + Unpin,
{
    type Item = Result<Packet, StreamError<WsError>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // poll the underlying websocket stream
            if let Some(msg) = ready!(Pin::new(&mut self.stream).poll_next(cx)) {
                match msg {
                    Ok(msg) => {
                        if msg.is_binary() {
                            // append data to the end of the buffer
                            self.read_buffer.extend(msg.into_data());
                            // parse the message
                            match Packet::parse(&self.read_buffer) {
                                IncompleteResult::Ok((remaining, pack)) => {
                                    debug!("packet parsed, {} bytes remaining", remaining.len());

                                    // remove parsed bytes
                                    let consume_len = self.read_buffer.len() - remaining.len();
                                    drop(self.read_buffer.drain(..consume_len));

                                    return Poll::Ready(Some(Ok(pack)));
                                }
                                IncompleteResult::Incomplete(needed) => {
                                    debug!("incomplete packet, {:?} needed", needed);
                                }
                                IncompleteResult::Err(e) => {
                                    warn!("error occurred when parsing incoming packet");
                                    return Poll::Ready(Some(Err(e.into())));
                                }
                            }
                        } else {
                            debug!("not a binary message, dropping");
                        }
                    }
                    Err(e) => {
                        // underlying websocket error, closing connection
                        warn!("error occurred when receiving message: {:?}", e);
                        return Poll::Ready(None);
                    }
                }
            } else {
                // underlying websocket closing
                return Poll::Ready(None);
            }
        }
    }
}

impl<T> Sink<Packet> for CodecStream<T>
where
    T: Sink<Message, Error = WsError> + Unpin,
{
    type Error = StreamError<WsError>;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.stream)
            .poll_ready(cx)
            .map_err(StreamError::from_ws_error)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Packet) -> Result<(), Self::Error> {
        Pin::new(&mut self.stream)
            .start_send(Message::binary(item.encode()))
            .map_err(StreamError::from_ws_error)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.stream)
            .poll_flush(cx)
            .map_err(StreamError::from_ws_error)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.stream)
            .poll_close(cx)
            .map_err(StreamError::from_ws_error)
    }
}
