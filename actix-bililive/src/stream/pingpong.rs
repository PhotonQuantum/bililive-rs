use std::pin::Pin;
use std::sync::Arc;
use std::task::Waker;
use std::task::{Context, Poll};

use awc::error::WsClientError;
use futures::Stream;
use futures::{ready, Sink};
use log::debug;

use crate::core::errors::Stream as StreamError;
use crate::core::packet::Packet;
use crate::core::stream::waker::WakerProxy;

use super::PacketOrPing;

pub struct PingPong<T> {
    stream: T,
    tx_waker: Arc<WakerProxy>,
}

impl<T> PingPong<T> {
    pub fn new(stream: T) -> Self {
        Self {
            stream,
            tx_waker: Arc::new(WakerProxy::default()),
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

impl<T> Stream for PingPong<T>
where
    T: Stream<Item = Result<PacketOrPing, StreamError<WsClientError>>>
        + Sink<PacketOrPing, Error = StreamError<WsClientError>>
        + Unpin,
{
    type Item = Result<Packet, StreamError<WsClientError>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // register current task to be waken on poll_ready
        self.tx_waker.rx(cx.waker());

        // ensure that all pending write op are completed
        ready!(self.with_context(|cx, s| Pin::new(s).poll_ready(cx)))?;

        match ready!(Pin::new(&mut self.stream).poll_next(cx)) {
            Some(Ok(PacketOrPing::PingPong(bytes))) => {
                // we need to send pong, so push it into the sink
                debug!("sending pong");
                Pin::new(&mut self.stream).start_send(PacketOrPing::PingPong(bytes))?;

                // ensure that pong is sent
                let _ = self.with_context(|cx, s| Pin::new(s).poll_flush(cx))?;

                Poll::Pending
            }
            Some(Ok(PacketOrPing::Packet(pack))) => Poll::Ready(Some(Ok(pack))),
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            None => Poll::Ready(None),
        }
    }
}

impl<T> Sink<Packet> for PingPong<T>
where
    T: Sink<PacketOrPing, Error = StreamError<WsClientError>> + Unpin,
{
    type Error = StreamError<WsClientError>;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // wake current task and stream task
        self.tx_waker.tx(cx.waker());

        // poll the underlying websocket sink
        self.with_context(|cx, s| Pin::new(s).poll_ready(cx))
    }

    fn start_send(mut self: Pin<&mut Self>, item: Packet) -> Result<(), Self::Error> {
        Pin::new(&mut self.stream).start_send(item.into())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // wake current task and stream task
        self.tx_waker.tx(cx.waker());

        // poll the underlying websocket sink
        self.with_context(|cx, s| Pin::new(s).poll_flush(cx))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // wake current task and stream task
        self.tx_waker.tx(cx.waker());

        // poll the underlying websocket sink
        self.with_context(|cx, s| Pin::new(s).poll_close(cx))
    }
}
