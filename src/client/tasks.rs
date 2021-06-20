use std::sync::Arc;

use futures_util::future::Either;
use futures_util::{future, SinkExt, StreamExt};
use nom::Err;
use pin_utils::pin_mut;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::tungstenite::Message;

use crate::errors::{ParseError, Result};
use crate::packet::{raw::RawPacket, Operation, Packet, Protocol};

use super::Client;
use super::{ChannelType, RxType, TxType};
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;

impl Client {
    pub(crate) async fn tx_task(
        mut tx: TxType,
        mut channel: mpsc::Receiver<ChannelType>,
        mut kill: broadcast::Receiver<()>,
    ) {
        loop {
            let channel_fut = channel.recv();
            let kill_fut = kill.recv();
            pin_mut!(channel_fut);
            pin_mut!(kill_fut);
            match future::select(channel_fut, kill_fut).await {
                Either::Left((Some((frame, notify_tx)), _)) => {
                    println!("sending packet");
                    let result = tx.send(frame).await;
                    println!("packet sent, notifying caller");
                    let _ = notify_tx.send(result.map_err(|e| e.into()));
                }
                _ => break,
            }
        }
    }
    pub(crate) async fn heart_beat_task(
        channel: mpsc::Sender<ChannelType>,
        mut kill: broadcast::Receiver<()>,
    ) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        let packet = RawPacket::new(Operation::HeartBeat, Protocol::Json, vec![]);
        let frame = Message::binary(packet);
        loop {
            let interval_fut = interval.tick();
            let kill_fut = kill.recv();
            pin_mut!(interval_fut);
            pin_mut!(kill_fut);
            match future::select(interval_fut, kill_fut).await {
                Either::Left(_) => {
                    let _ = Self::_send_frame(&channel, frame.clone()).await;
                }
                _ => break,
            }
        }
    }
    pub(crate) async fn rx_task(
        callback: Arc<dyn Fn(Packet) + Send + Sync>,
        mut rx: RxType,
        mut kill: broadcast::Receiver<()>,
        popularity: Arc<AtomicI32>,
    ) -> Result<()> {
        let mut buf = vec![];
        loop {
            let rx_fut = rx.next();
            let kill_fut = kill.recv();
            pin_mut!(rx_fut);
            pin_mut!(kill_fut);
            match future::select(rx_fut, kill_fut).await {
                Either::Left((Some(msg), _)) => {
                    match msg {
                        Ok(msg) => {
                            if msg.is_binary() {
                                buf.extend(msg.into_data());
                                match RawPacket::parse(&buf) {
                                    Ok((remaining, raw)) => {
                                        println!(
                                            "packet parsed, {} bytes remaining",
                                            remaining.len()
                                        );
                                        buf = remaining.to_vec(); // TODO to be optimized
                                        let pack = Packet::from(raw);
                                        if pack.op() == Operation::HeartBeatResponse {
                                            if let Ok(new_popularity) = pack.int32_be() {
                                                println!("update popularity {}", new_popularity);
                                                popularity.store(new_popularity, Relaxed);
                                            }
                                        }
                                        (*callback)(pack)
                                    }
                                    Err(Err::Incomplete(needed)) => {
                                        println!("incomplete packet, {:?} needed", needed);
                                    }
                                    Err(Err::Error(e) | Err::Failure(e)) => {
                                        return Err(
                                            ParseError::PacketError(format!("{:?}", e)).into()
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }
}
