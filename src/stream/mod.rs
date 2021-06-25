use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crossbeam::queue::ArrayQueue;
use futures::stream::{SplitSink, SplitStream};
use futures::{sink::Sink, stream::Stream};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::errors::StreamError;
use crate::packet::raw::RawPacket;
use crate::packet::Packet;
use crate::stream::channel::ConnEvent;
use crate::stream::config::StreamConfig;
use crate::stream::tasks::{conn_task, heart_beat_task, rx_task, tx_task};

use self::state::*;
use self::waker::*;
use tokio::task::JoinHandle;

mod channel;
mod config;
mod state;
mod tasks;
mod utils;
mod waker;

pub(crate) type WsTxType = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
pub(crate) type WsRxType = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub(crate) type ConnTxType = broadcast::Sender<ConnEvent>;
pub(crate) type ConnRxType = broadcast::Receiver<ConnEvent>;
pub(crate) type SinkTxType = mpsc::Sender<Message>;
pub(crate) type SinkRxType = mpsc::Receiver<Message>;
pub(crate) type Result<T> = std::result::Result<T, StreamError>;
type StdResult<T, E> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Handles {
    tx_task: JoinHandle<()>,
    rx_task: JoinHandle<()>,
    conn_task: JoinHandle<()>,
    heart_beat_task: JoinHandle<()>,
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
    config: StreamConfig,
    handles: Handles,
}

impl BililiveStream {
    pub fn new(config: &StreamConfig) -> Self {
        let state = Arc::new(StreamStateStore::new());
        let waker = Arc::new(WakerProxy::new());
        let rx_buffer = Arc::new(ArrayQueue::new(32));

        let (ws_tx_sender, ws_tx_receiver) = mpsc::channel(32);
        let (ws_rx_sender, ws_rx_receiver) = mpsc::channel(32);
        let (conn_tx, conn_rx) = broadcast::channel(32);
        let (tx_buffer_sender, tx_buffer_receiver) = mpsc::channel(32);

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
                config.clone(),
                ws_tx_sender,
                ws_rx_sender,
                (conn_tx, conn_rx),
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
            config: config.clone(),
            handles,
        }
    }

    fn room_enter_message(config: &StreamConfig) -> Value {
        json!({
            "uid": config.uid,
            "roomid": config.room_id,
            "protover": 2,
            "platform": "web",
            "clientver": "1.8.2",
            "type": 2,
            "key": config.token
        })
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
