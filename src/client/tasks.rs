use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use futures_util::future::Either;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::tungstenite::Message;

use crate::errors::Result;
use crate::packet::{raw::RawPacket, Operation, Packet, Protocol};
use crate::IncompleteResult;

use super::Client;
use super::{ChannelType, RxType, TxType};

impl Client {
    pub(crate) async fn tx_task(
        mut tx: TxType,
        mut channel: mpsc::Receiver<ChannelType>,
        mut kill: broadcast::Receiver<()>,
    ) {
        while_let_kill!(kill.recv(), channel.recv(), Some((frame, notify_tx)) => {
            println!("sending packet");
            let result = tx.send(frame).await;
            println!("packet sent, notifying caller");
            let _ = notify_tx.send(result.map_err(|e| e.into()));
        });
    }
    pub(crate) async fn heart_beat_task(
        channel: mpsc::Sender<ChannelType>,
        mut kill: broadcast::Receiver<()>,
    ) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        let packet = RawPacket::new(Operation::HeartBeat, Protocol::Json, vec![]);
        let frame = Message::binary(packet);
        while_let_kill!(kill.recv(), interval.tick(), _ => {
            let _ = Self::_send_frame(&channel, frame.clone()).await;
        });
    }
    pub(crate) async fn rx_task(
        callback: Arc<dyn Fn(Packet) + Send + Sync>,
        mut rx: RxType,
        mut kill: broadcast::Receiver<()>,
        popularity: Arc<AtomicI32>,
    ) -> Result<()> {
        let mut buf = vec![];
        while_let_kill!(kill.recv(), rx.next(), Some(msg) => {
            match msg {
                Ok(msg) => {
                    if msg.is_binary() {
                        buf.extend(msg.into_data());
                        match RawPacket::parse(&buf) {
                            IncompleteResult::Ok((remaining, raw)) => {
                                println!("packet parsed, {} bytes remaining", remaining.len());

                                let consume_len = buf.len() - remaining.len();
                                drop(buf.drain(..consume_len));

                                let pack = Packet::from(raw);
                                if pack.op() == Operation::HeartBeatResponse {
                                    if let Ok(new_popularity) = pack.int32_be() {
                                        println!("update popularity {}", new_popularity);
                                        popularity.store(new_popularity, Relaxed);
                                    }
                                }
                                (*callback)(pack)
                            }
                            IncompleteResult::Incomplete(needed) => {
                                println!("incomplete packet, {:?} needed", needed);
                            }
                            IncompleteResult::Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                }
                Err(e) => return Err(e.into()),
            }
        });
        Ok(())
    }
}
