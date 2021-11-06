//! Bilibili live stream.
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

use async_tungstenite::tungstenite::{error::Error as WsError, Message};
use futures::{ready, Sink, Stream};
use log::{debug, warn};

use crate::errors::{BililiveError, IncompleteResult};
use crate::packet::{Operation, Packet, Protocol};

use self::waker::WakerProxy;

pub(crate) mod retry;
mod utils;
mod waker;

#[cfg(test)]
mod tests;

type StreamResult<T> = std::result::Result<T, BililiveError>;

/// A wrapper around an underlying websocket stream (w/o retry) which implements bilibili live protocol.
///
/// A `BililiveStream<T>` represents a stream that has finished basic handshake procedure, and exposes
/// `Stream` and `Sink` interfaces. Heartbeats are automatically handled as long as the stream is
/// polled frequently.
pub struct BililiveStream<T> {
    // underlying websocket stream
    stream: T,
    // waker proxy for tx, see WakerProxy for details
    tx_waker: Arc<WakerProxy>,
    // last time when heart beat is sent
    last_hb: Option<Instant>,
    // rx buffer
    read_buffer: Vec<u8>,
}

impl<T> BililiveStream<T> {
    /// Convert a raw stream into a [`BililiveStream`](BililiveStream) without performing websocket protocol establishment.
    ///
    /// You may want to use `connect` or `connect_with_retry` in [`connect`](crate::connect) module instead.
    pub fn from_raw_stream(stream: T) -> Self {
        Self {
            stream,
            tx_waker: Arc::new(Default::default()),
            last_hb: None,
            read_buffer: vec![],
        }
    }

    fn with_context<F, U>(&mut self, f: F) -> U
    where
        F: FnOnce(&mut Context<'_>, &mut T) -> U,
    {
        let waker = Waker::from(self.tx_waker.clone());
        let mut cx = Context::from_waker(&waker);

        f(&mut cx, &mut self.stream)
    }
}

impl<T> Stream for BililiveStream<T>
where
    T: Stream<Item = Result<Message, WsError>> + Sink<Message, Error = WsError> + Unpin,
{
    type Item = StreamResult<Packet>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // register current task to be waken on poll_ready
        self.tx_waker.rx(cx.waker());

        // ensure that all pending write op are completed
        ready!(self.with_context(|cx, s| Pin::new(s).poll_ready(cx)))?;

        // check whether we need to send heartbeat now.
        let now = Instant::now();
        let need_hb = self
            .last_hb
            .map_or(true, |last_hb| now - last_hb >= Duration::from_secs(30));

        if need_hb {
            // we need to send heartbeat, so push it into the sink
            debug!("sending heartbeat");
            self.as_mut()
                .start_send(Packet::new(Operation::HeartBeat, Protocol::Json, vec![]))?;

            // Update the time we sent the heartbeat.
            // It must be earlier than other non-blocking op so that heartbeat
            // won't be sent repeatedly.
            self.last_hb = Some(now);

            // Schedule current task to be waken in case there's no incoming
            // websocket message in a long time.
            #[cfg(feature = "tokio")]
            {
                let waker = cx.waker().clone();
                tokio::spawn(async {
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    waker.wake();
                });
            }
            #[cfg(feature = "async-std")]
            {
                let waker = cx.waker().clone();
                async_std::task::spawn(async {
                    async_std::task::sleep(Duration::from_secs(30)).await;
                    waker.wake();
                });
            }

            // ensure that heartbeat is sent
            ready!(self.with_context(|cx, s| Pin::new(s).poll_flush(cx)))?;
        }

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
                                    return Poll::Ready(Some(Err(e)));
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

impl<T> Sink<Packet> for BililiveStream<T>
where
    T: Sink<Message, Error = WsError> + Unpin,
{
    type Error = BililiveError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // wake current task and stream task
        self.tx_waker.tx(cx.waker());
        let waker = Waker::from(self.tx_waker.clone());
        let mut cx = Context::from_waker(&waker);

        // poll the underlying websocket sink
        Poll::Ready(Ok(ready!(Pin::new(&mut self.stream).poll_ready(&mut cx))?))
    }

    fn start_send(mut self: Pin<&mut Self>, item: Packet) -> Result<(), Self::Error> {
        Ok(Pin::new(&mut self.stream).start_send(Message::binary(item.encode()))?)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // wake current task and stream task
        self.tx_waker.tx(cx.waker());
        let waker = Waker::from(self.tx_waker.clone());
        let mut cx = Context::from_waker(&waker);

        // poll the underlying websocket sink
        Poll::Ready(Ok(ready!(Pin::new(&mut self.stream).poll_flush(&mut cx))?))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // wake current task and stream task
        self.tx_waker.tx(cx.waker());
        let waker = Waker::from(self.tx_waker.clone());
        let mut cx = Context::from_waker(&waker);

        // poll the underlying websocket sink
        Poll::Ready(Ok(ready!(Pin::new(&mut self.stream).poll_close(&mut cx))?))
    }
}
