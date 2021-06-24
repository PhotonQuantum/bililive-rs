use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crossbeam::queue::ArrayQueue;
use futures::stream::{SplitSink, SplitStream};
use futures::{sink::Sink, stream::Stream};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::errors::StreamError;
use crate::packet::raw::RawPacket;
use crate::packet::Packet;
use crate::stream::channel::ConnEvent;

use self::state::*;
use self::waker::*;

mod channel;
mod state;
mod tasks;
mod waker;

pub(crate) type WsTxType = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
pub(crate) type WsRxType = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub(crate) type ConnTxType = broadcast::Sender<ConnEvent>;
pub(crate) type ConnRxType = broadcast::Receiver<ConnEvent>;
pub(crate) type SinkTxType = mpsc::Sender<Message>;
pub(crate) type SinkRxType = mpsc::Receiver<Message>;
pub(crate) type Result<T> = std::result::Result<T, StreamError>;
type StdResult<T, E> = std::result::Result<T, E>;

pub struct BililiveStream {
    waker: WakerProxy,
    // rx/tx wakers
    state: StreamStateStore,
    // connection state
    rx_buffer: Arc<ArrayQueue<Result<Packet>>>,
    // buffer for incoming packets
    tx_sender: mpsc::Sender<Message>, // sender of tx channel (receiver is in TxTask)
}

impl BililiveStream {
    pub fn new() -> Self {
        Self {
            waker: Default::default(),
            state: Default::default(),
            rx_buffer: Arc::new(ArrayQueue::new(32)),
            tx_sender: todo!(),
        }
    }
}

impl Stream for BililiveStream {
    type Item = Result<Packet>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.state.load() {
            StreamState::Active => {
                if let Some(item) = self.rx_buffer.pop() {
                    Poll::Ready(Some(item))
                } else {
                    self.waker.register(WakeMode::Rx, cx.waker());
                    Poll::Pending
                }
            }
            StreamState::Establishing => {
                self.waker.register(WakeMode::Rx, cx.waker());
                Poll::Pending
            }
            StreamState::Terminated => Poll::Ready(None),
        }
    }
}

impl Sink<Packet> for BililiveStream {
    type Error = StreamError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<StdResult<(), Self::Error>> {
        match self.state.load() {
            StreamState::Active => Poll::Ready(Ok(())),
            StreamState::Establishing => {
                self.waker.register(WakeMode::Tx, cx.waker());
                Poll::Pending
            }
            StreamState::Terminated => Poll::Ready(Err(StreamError::AlreadyClosed)),
        }
    }

    fn start_send(self: Pin<&mut Self>, item: Packet) -> StdResult<(), Self::Error> {
        let raw_packet = RawPacket::from(item);
        let frame = Message::binary(raw_packet);
        self.tx_sender
            .blocking_send(frame)
            .map_err(|_| StreamError::AlreadyClosed)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<StdResult<(), Self::Error>> {
        if self.tx_sender.capacity() == 32 {
            // Tx buffer is empty.
            Poll::Ready(Ok(()))
        } else {
            match self.state.load() {
                StreamState::Active => {
                    // Buffer is not empty, and connection is up.
                    // The TxTask is processing the packets.
                    self.waker.register(WakeMode::Tx, cx.waker());
                    Poll::Pending
                }
                StreamState::Establishing => {
                    // Connection is being established.
                    self.waker.register(WakeMode::Tx, cx.waker());
                    Poll::Pending
                }
                StreamState::Terminated => {
                    // Connection is terminated and the sink can no longer send any packet.
                    Poll::Ready(Err(StreamError::AlreadyClosed))
                }
            }
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<StdResult<(), Self::Error>> {
        self.poll_flush(cx)
    }
}
