use std::pin::Pin;
use std::sync::Arc;
use std::task::Waker;
use std::task::{Context, Poll};

use awc::error::WsProtocolError;
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
}

impl<T> Stream for PingPong<T>
where
    T: Stream<Item = Result<PacketOrPing, StreamError<WsProtocolError>>>
        + Sink<PacketOrPing, Error = StreamError<WsProtocolError>>
        + Unpin,
{
    type Item = Result<Packet, StreamError<WsProtocolError>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // register current task to be waken on poll_ready
        self.tx_waker.rx(cx.waker());

        // ensure that all pending write op are completed
        ready!(Sink::<PacketOrPing>::poll_ready(self.as_mut(), cx))?;

        match ready!(Pin::new(&mut self.stream).poll_next(cx)) {
            Some(Ok(PacketOrPing::PingPong(bytes))) => {
                // we need to send pong, so push it into the sink
                debug!("sending pong");
                self.as_mut().start_send(PacketOrPing::PingPong(bytes))?;

                // ensure that pong is sent
                let _ = Sink::<PacketOrPing>::poll_flush(self.as_mut(), cx)?;

                Poll::Pending
            }
            Some(Ok(PacketOrPing::Packet(pack))) => Poll::Ready(Some(Ok(pack))),
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            None => Poll::Ready(None),
        }
    }
}

impl<T> Sink<PacketOrPing> for PingPong<T>
where
    T: Sink<PacketOrPing, Error = StreamError<WsProtocolError>> + Unpin,
{
    type Error = StreamError<WsProtocolError>;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // wake current task and stream task
        self.tx_waker.tx(cx.waker());
        let waker = Waker::from(self.tx_waker.clone());
        let mut cx = Context::from_waker(&waker);

        // poll the underlying websocket sink
        Pin::new(&mut self.stream).poll_ready(&mut cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: PacketOrPing) -> Result<(), Self::Error> {
        Pin::new(&mut self.stream).start_send(item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // wake current task and stream task
        self.tx_waker.tx(cx.waker());
        let waker = Waker::from(self.tx_waker.clone());
        let mut cx = Context::from_waker(&waker);

        // poll the underlying websocket sink
        Pin::new(&mut self.stream).poll_flush(&mut cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // wake current task and stream task
        self.tx_waker.tx(cx.waker());
        let waker = Waker::from(self.tx_waker.clone());
        let mut cx = Context::from_waker(&waker);

        // poll the underlying websocket sink
        Pin::new(&mut self.stream).poll_close(&mut cx)
    }
}

impl<T> Sink<Packet> for PingPong<T>
where
    Self: Sink<PacketOrPing, Error = StreamError<WsProtocolError>> + Unpin,
{
    type Error = StreamError<WsProtocolError>;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::<PacketOrPing>::poll_ready(self, cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Packet) -> Result<(), Self::Error> {
        Sink::<PacketOrPing>::start_send(self, item.into())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::<PacketOrPing>::poll_flush(self, cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::<PacketOrPing>::poll_close(self, cx)
    }
}
