use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

use async_tungstenite::tungstenite::{error::Error as WsError, Message};
use futures::{ready, Sink, Stream};
use log::{debug, warn};
use once_cell::sync::Lazy;

use crate::errors::{BililiveError, IncompleteResult};
use crate::packet::raw::RawPacket;
use crate::packet::{Operation, Packet, Protocol};

use self::waker::*;

pub mod retry;
mod utils;
mod waker;

#[cfg(test)]
mod tests;

type StreamResult<T> = std::result::Result<T, BililiveError>;

static HB_MSG: Lazy<Packet> =
    Lazy::new(|| Packet::new(Operation::HeartBeat, Protocol::Json, vec![]));

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
    pub fn new(stream: T) -> Self {
        Self {
            stream,
            tx_waker: Arc::new(Default::default()),
            last_hb: None,
            read_buffer: vec![],
        }
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
        ready!(self.as_mut().poll_ready(cx))?;

        // check whether we need to send heartbeat now.
        let now = Instant::now();
        let need_hb = self
            .last_hb
            .map_or(true, |last_hb| now - last_hb >= Duration::from_secs(30));

        if need_hb {
            // we need to send heartbeat, so push it into the sink
            debug!("sending heartbeat");
            self.as_mut().start_send(HB_MSG.clone())?;

            // Update the time we sent the heartbeat.
            // It must be earlier than other non-blocking op so that heartbeat
            // won't be sent repeatedly.
            self.last_hb = Some(now);

            // Schedule current task to be waken in case there's no incoming
            // websocket message in a long time.
            debug!("scheduling task awake");
            #[cfg(feature = "tokio")]
            {
                let waker = cx.waker().clone();
                tokio::spawn(async {
                    debug!("awake task online");
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    debug!("waking read task");
                    waker.wake();
                });
            }
            #[cfg(feature = "async-std")]
            {
                let waker = cx.waker().clone();
                async_std::task::spawn(async {
                    debug!("awake task online");
                    async_std::task::sleep(Duration::from_secs(30)).await;
                    debug!("waking read task");
                    waker.wake();
                });
            }
        }

        // ensure that heartbeat is sent
        ready!(self.as_mut().poll_flush(cx))?;

        loop {
            // poll the underlying websocket stream
            if let Some(msg) = ready!(Pin::new(&mut self.stream).poll_next(cx)) {
                match msg {
                    Ok(msg) => {
                        if msg.is_binary() {
                            // append data to the end of the buffer
                            self.read_buffer.extend(msg.into_data());
                            // parse the message
                            match RawPacket::parse(&self.read_buffer) {
                                IncompleteResult::Ok((remaining, raw)) => {
                                    debug!("packet parsed, {} bytes remaining", remaining.len());

                                    // remove parsed bytes
                                    let consume_len = self.read_buffer.len() - remaining.len();
                                    drop(self.read_buffer.drain(..consume_len));

                                    let pack = Packet::from(raw);
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
        Ok(Pin::new(&mut self.stream).start_send(Message::binary(RawPacket::from(item)))?)
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
