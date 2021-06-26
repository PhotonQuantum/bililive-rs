use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crossbeam::queue::ArrayQueue;
use futures::stream::{SplitSink, SplitStream};
use futures::{sink::Sink, stream::Stream};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub use crate::config::*;
use crate::errors::StreamError;
use crate::packet::raw::RawPacket;
use crate::packet::Packet;

use self::state::*;
use self::tasks::{conn_task, heart_beat_task, rx_task, tx_task};
use self::waker::*;

mod channel;
mod state;
mod tasks;
mod utils;
mod waker;

type WsTxType = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type WsRxType = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
type ConnTxType = broadcast::Sender<ConnEvent>;
type ConnRxType = broadcast::Receiver<ConnEvent>;
type SinkTxType = mpsc::Sender<Message>;
type SinkRxType = mpsc::Receiver<Message>;
type Result<T> = std::result::Result<T, StreamError>;
type StdResult<T, E> = std::result::Result<T, E>;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ConnEvent {
    Close,
    Failure,
}

#[derive(Debug)]
pub struct Handles {
    tx_task: JoinHandle<()>,
    rx_task: JoinHandle<()>,
    conn_task: JoinHandle<()>,
    heart_beat_task: JoinHandle<()>,
}

impl Handles {
    pub async fn join(&mut self) {
        (&mut self.tx_task).await.unwrap();
        (&mut self.rx_task).await.unwrap();
        (&mut self.conn_task).await.unwrap();
        (&mut self.heart_beat_task).await.unwrap();
    }
}

#[derive(Debug)]
pub struct BililiveStream {
    waker: Arc<WakerProxy>,
    // rx/tx wakers
    state: Arc<StreamStateStore>,
    // connection state
    rx_buffer: Arc<ArrayQueue<Result<Packet>>>,
    // buffer for incoming packets
    tx_sender: mpsc::Sender<Message>,
    // sender of tx channel (receiver is in TxTask)
    tx_buffer_cap: usize,
    conn_tx: broadcast::Sender<ConnEvent>,
    handles: Handles,
}

impl BililiveStream {
    pub fn new(config: StreamConfig) -> Self {
        let state = Arc::new(StreamStateStore::new());
        let waker = Arc::new(WakerProxy::new());
        let rx_buffer = Arc::new(ArrayQueue::new(config.buffer.rx_buffer));

        let (ws_tx_sender, ws_tx_receiver) = mpsc::channel(config.buffer.socket_buffer);
        let (ws_rx_sender, ws_rx_receiver) = mpsc::channel(config.buffer.socket_buffer);
        let (conn_tx, conn_rx) = broadcast::channel(config.buffer.conn_event_buffer);
        let (tx_buffer_sender, tx_buffer_receiver) = mpsc::channel(config.buffer.tx_buffer);

        let tx_buffer_cap = config.buffer.tx_buffer;

        let tx_task = {
            let conn_tx = conn_tx.clone();
            let conn_rx = conn_tx.subscribe();
            tx_task(
                ws_tx_receiver,
                tx_buffer_receiver,
                (conn_tx, conn_rx),
                state.clone(),
                waker.clone(),
            )
        };
        let rx_task = {
            let conn_tx = conn_tx.clone();
            let conn_rx = conn_tx.subscribe();
            rx_task(
                ws_rx_receiver,
                rx_buffer.clone(),
                (conn_tx, conn_rx),
                state.clone(),
                waker.clone(),
            )
        };
        let conn_task = {
            let conn_rx = conn_tx.subscribe();
            conn_task(
                config,
                ws_tx_sender,
                ws_rx_sender,
                (conn_tx.clone(), conn_rx),
                state.clone(),
                waker.clone(),
            )
        };
        let heart_beat_task = heart_beat_task(tx_buffer_sender.clone(), conn_rx, state.clone());

        let handles = Handles {
            tx_task: tokio::spawn(tx_task),
            rx_task: tokio::spawn(rx_task),
            conn_task: tokio::spawn(conn_task),
            heart_beat_task: tokio::spawn(heart_beat_task),
        };

        Self {
            waker,
            state,
            rx_buffer,
            tx_sender: tx_buffer_sender,
            tx_buffer_cap,
            conn_tx,
            handles,
        }
    }

    pub fn close(&self) {
        self.conn_tx.send(ConnEvent::Close).unwrap();
    }

    pub async fn join(&mut self) {
        self.handles.join().await;
    }
}

impl Stream for BililiveStream {
    type Item = Result<Packet>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.state.load() {
            StreamState::Active => self.rx_buffer.pop().map_or_else(
                || {
                    self.waker.register(WakeMode::Rx, cx.waker());
                    Poll::Pending
                },
                |item| Poll::Ready(Some(item)),
            ),
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
        if self.tx_sender.capacity() == self.tx_buffer_cap {
            // Tx buffer is empty.
            Poll::Ready(Ok(()))
        } else {
            match self.state.load() {
                StreamState::Active | StreamState::Establishing => {
                    // Buffer is not empty, and connection is up.
                    // The TxTask is processing the packets.
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
