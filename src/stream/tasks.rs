use std::sync::Arc;

use crossbeam::queue::ArrayQueue;
use futures::future::Either;
use futures::{SinkExt, StreamExt};
use futures_util::future::select;
use tokio::sync::{mpsc, oneshot};
use tokio::time::Duration;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::raw::RawPacket;
use crate::stream::state::{StreamState, StreamStateStore};
use crate::stream::waker::{WakeMode, WakerProxy};
use crate::{
    ConnRxType, ConnTxType, IncompleteResult, Operation, Packet, Protocol, SinkRxType, SinkTxType,
    WsRxType, WsTxType,
};

use super::channel::ConnEvent;
use super::Result;
use crate::stream::config::StreamConfig;
use crate::stream::utils::room_enter_message;
use std::time::Instant;

// tx_buffer: tx message buffer
// conn_rx: connection event rx
// conn_state: stream connection state
pub(crate) async fn heart_beat_task(
    tx_buffer: SinkTxType,
    mut conn_rx: ConnRxType,
    conn_state: Arc<StreamStateStore>,
) {
    let mut ticker = tokio::time::interval(Duration::from_secs(30));
    let msg = Message::binary(RawPacket::new(Operation::HeartBeat, Protocol::Json, vec![]));
    loop {
        let fut = ticker.tick();
        let conn_fut = conn_rx.recv();
        tokio::pin!(fut);
        tokio::pin!(conn_fut);
        match select(fut, conn_fut).await {
            Either::Left(_) => {
                if conn_state.load().is_active() {
                    tx_buffer.send(msg.clone()).await.unwrap();
                }
            }
            Either::Right((Ok(event), _)) => match event {
                ConnEvent::Close => return,
                ConnEvent::Failure => continue,
            },
            _ => {}
        }
    }
}

// servers: danmaku servers
// ws_tx_sender: websocket tx will be sent through this channel (a receipt sender is attached)
// ws_rx_sender: websocket rx will be sent through this channel (a receipt sender is attached)
// conn_(tx, rx): connection event channel
// conn_state: stream connection state
// waker: waker proxy
pub(crate) async fn conn_task(
    config: StreamConfig,
    ws_tx_sender: mpsc::Sender<(WsTxType, oneshot::Sender<()>)>,
    ws_rx_sender: mpsc::Sender<(WsRxType, oneshot::Sender<()>)>,
    conn: (ConnTxType, ConnRxType),
    conn_state: Arc<StreamStateStore>,
    waker: Arc<WakerProxy>,
) {
    let (conn_tx, mut conn_rx) = conn;
    let mut servers = config.servers.iter().cycle();

    let mut last_connect = None;
    let mut fail_count: u32 = 0;
    loop {
        if let Some(last_connect) = last_connect {
            let now = Instant::now();
            if now - last_connect < config.retry.min_conn_duration {
                fail_count += 1;
            } else {
                fail_count = 1;
            }
            let maybe_delay = (*config.retry.retry_policy)(fail_count);
            if let Some(delay) = maybe_delay {
                tokio::time::sleep(delay).await
            } else {
                // disconnect
                todo!()
            }
        }

        last_connect = Some(Instant::now());

        let server = servers.next().unwrap();
        let (ws, _) = connect_async(server).await.unwrap();
        let (mut tx, rx) = ws.split();

        // send room enter message before putting it to tx task
        if tx.send(room_enter_message(&config)).await.is_err() {
            continue;
        }

        let (tx_receipt_sender, tx_receipt_receiver) = oneshot::channel();
        let (rx_receipt_sender, rx_receipt_receiver) = oneshot::channel();
        assert!(ws_tx_sender.send((tx, tx_receipt_sender)).await.is_ok());
        assert!(ws_rx_sender.send((rx, rx_receipt_sender)).await.is_ok());
        let (receipt_tx, receipt_rx) = tokio::join!(tx_receipt_receiver, rx_receipt_receiver);
        receipt_tx.unwrap();
        receipt_rx.unwrap();

        conn_state.store(StreamState::Active);
        // ready to send message
        waker.wake(WakeMode::Tx);

        match conn_rx.recv().await {
            Ok(ConnEvent::Failure) => continue,
            _ => break,
        }
    }
}

// ws_tx_receiver: websocket tx will be received through this channel (a receipt should be sent when received)
// conn_(tx, rx): connection event channel
// tx_buffer: tx message buffer
// conn_state: stream connection state
// waker: waker proxy
pub(crate) async fn tx_task(
    mut ws_tx_receiver: mpsc::Receiver<(WsTxType, oneshot::Sender<()>)>,
    mut tx_buffer: SinkRxType,
    conn: (ConnTxType, ConnRxType),
    conn_state: Arc<StreamStateStore>,
    waker: Arc<WakerProxy>,
) {
    let (conn_tx, mut conn_rx) = conn;

    loop {
        let (mut ws_tx, receipt) = ws_tx_receiver.recv().await.unwrap();
        receipt.send(()).unwrap();
        let mut drop_conn_rx_once;
        loop {
            let fut = tx_buffer.recv();
            let conn_fut = conn_rx.recv();
            tokio::pin!(fut);
            tokio::pin!(conn_fut);
            drop_conn_rx_once = false;
            match select(fut, conn_fut).await {
                Either::Left((Some(msg), _)) => {
                    let send_result = ws_tx.send(msg).await;
                    if send_result.is_err() {
                        // connection failed
                        if conn_state.load().is_active() {
                            conn_state.store(StreamState::Establishing);
                            conn_tx.send(ConnEvent::Failure).unwrap();
                            drop_conn_rx_once = true;   // drop the signal just sent
                        }
                        break;
                    } else {
                        waker.wake(WakeMode::Tx);
                    }
                }
                Either::Right((Ok(event), _)) => match event {
                    ConnEvent::Close => return,
                    ConnEvent::Failure => break,
                },
                _ => {
                    // connection failed
                    if conn_state.load().is_active() {
                        conn_state.store(StreamState::Establishing);
                        conn_tx.send(ConnEvent::Failure).unwrap();
                        drop_conn_rx_once = true;   // drop the signal just sent
                    }
                    break;
                }
            }
        }
        if drop_conn_rx_once {
            conn_rx.recv().await.unwrap();
        }
    }
}

// ws_rx_receiver: websocket rx will be received through this channel (a receipt should be sent when received)
// conn_(tx, rx): connection event channel
// tx_buffer: rx packet buffer
// conn_state: stream connection state
// waker: waker proxy
pub(crate) async fn rx_task(
    mut ws_rx_receiver: mpsc::Receiver<(WsRxType, oneshot::Sender<()>)>,
    rx_buffer: Arc<ArrayQueue<Result<Packet>>>,
    conn: (ConnTxType, ConnRxType),
    conn_state: Arc<StreamStateStore>,
    waker: Arc<WakerProxy>,
) {
    let (conn_tx, mut conn_rx) = conn;

    loop {
        let (mut ws_rx, receipt) = ws_rx_receiver.recv().await.unwrap();
        receipt.send(()).unwrap();
        let mut drop_conn_rx_once;
        let mut buf = vec![];
        loop {
            let fut = ws_rx.next();
            let conn_fut = conn_rx.recv();
            tokio::pin!(fut);
            tokio::pin!(conn_fut);
            drop_conn_rx_once = false;
            match select(fut, conn_fut).await {
                Either::Left((Some(Ok(msg)), _)) => {
                    if msg.is_binary() {
                        buf.extend(msg.into_data());
                        match RawPacket::parse(&buf) {
                            IncompleteResult::Ok((remaining, raw)) => {
                                println!("packet parsed, {} bytes remaining", remaining.len());

                                let consume_len = buf.len() - remaining.len();
                                drop(buf.drain(..consume_len));

                                let pack = Packet::from(raw);
                                rx_buffer.push(Ok(pack)).unwrap();

                                waker.wake(WakeMode::Rx);
                            }
                            IncompleteResult::Incomplete(needed) => {
                                println!("incomplete packet, {:?} needed", needed);
                            }
                            IncompleteResult::Err(_) => {
                                // connection failed
                                if conn_state.load().is_active() {
                                    conn_state.store(StreamState::Establishing);
                                    conn_tx.send(ConnEvent::Failure).unwrap();
                                    drop_conn_rx_once = true;   // drop the signal just sent
                                }
                                break;
                            }
                        }
                    }
                }
                Either::Right((Ok(event), _)) => match event {
                    ConnEvent::Close => return,
                    ConnEvent::Failure => break,
                },
                _ => {
                    // connection failed
                    if conn_state.load().is_active() {
                        conn_state.store(StreamState::Establishing);
                        conn_tx.send(ConnEvent::Failure).unwrap();
                        drop_conn_rx_once = true;   // drop the signal just sent
                    }
                    break;
                }
            }
        }
        if drop_conn_rx_once {
            conn_rx.recv().await.unwrap();
        }
    }
}
