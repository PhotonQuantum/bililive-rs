use std::borrow::BorrowMut;
use std::sync::Arc;

use futures_util::future;
use futures_util::future::Either;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use nom::Err;
use pin_utils::pin_mut;
use serde_json::json;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::errors::{BililiveError, Result};
use crate::packet::{raw::RawPacket, Operation, Protocol};
use crate::{Packet, ParseError};

type TxType = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type RxType = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
type ChannelType = (Message, oneshot::Sender<Result<()>>);

struct JoinHandles {
    tx_handle: JoinHandle<()>,
    rx_handle: JoinHandle<Result<()>>,
    heartbeat_handle: JoinHandle<()>,
}

impl JoinHandles {
    pub fn new(
        tx_handle: JoinHandle<()>,
        rx_handle: JoinHandle<Result<()>>,
        heartbeat_handle: JoinHandle<()>,
    ) -> Self {
        JoinHandles {
            tx_handle,
            rx_handle,
            heartbeat_handle,
        }
    }

    pub async fn join(&mut self) -> Result<()> {
        self.heartbeat_handle.borrow_mut().await?;
        self.tx_handle.borrow_mut().await?;
        self.rx_handle.borrow_mut().await?
    }
}

pub struct Client {
    http: reqwest::Client,
    room_id: u64,
    uid: u64,
    compression: bool,
    tx_buffer: usize,
    // pub(crate) rx: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    callback: Arc<dyn Fn(Packet) + Send + Sync>,
    handles: Option<JoinHandles>,
    tx_channel: Option<mpsc::Sender<ChannelType>>,
    kill: broadcast::Sender<()>,
}

impl Client {
    pub(crate) fn new(
        http: reqwest::Client,
        room_id: u64,
        uid: u64,
        compression: bool,
        tx_buffer: usize,
        callback: Arc<dyn Fn(Packet) + Send + Sync>,
    ) -> Self {
        let (kill, _) = broadcast::channel(32);
        Self {
            http,
            room_id,
            uid,
            compression,
            tx_buffer,
            callback,
            handles: None,
            tx_channel: None,
            kill,
        }
    }
    pub fn room_id(&self) -> u64 {
        self.room_id
    }
    pub async fn connect(&mut self) -> Result<()> {
        let (stream, _) = connect_async("wss://broadcastlv.chat.bilibili.com/sub").await?;
        let (tx, rx) = stream.split();

        let (tx_chan, rx_chan) = mpsc::channel(self.tx_buffer);

        self.tx_channel = Some(tx_chan.clone());

        println!("spawning tasks");
        let tx_handle = tokio::spawn(Self::tx_task(tx, rx_chan, self.kill.subscribe()));
        let rx_handle = tokio::spawn(Self::rx_task(
            self.callback.clone(),
            rx,
            self.kill.subscribe(),
        ));

        self.enter_room().await?;

        let heartbeat_handle = tokio::spawn(Self::heart_beat_task(
            tx_chan,
            self.kill.subscribe(),
        ));
        self.handles = Some(JoinHandles::new(tx_handle, rx_handle, heartbeat_handle));

        Ok(())
    }
    pub async fn close(&mut self) -> Result<()> {
        println!("closing");
        self.send_frame(Message::Close(None)).await?;

        println!("sending kill signal");
        self.kill.send(()).unwrap();

        let mut handles = self.handles.take().unwrap();
        self.tx_channel = None;

        handles.join().await?;

        Ok(())
    }
    pub async fn join(&mut self) -> Result<()> {
        if let Some(handles) = self.handles.as_mut() {
            handles.join().await?;
        }

        Ok(())
    }
    async fn send_frame(&self, frame: Message) -> Result<()> {
        let channel = self
            .tx_channel
            .as_ref()
            .ok_or(BililiveError::NotConnected)?;
        Self::_send_frame(channel, frame).await
    }
    async fn _send_frame(channel: &mpsc::Sender<ChannelType>, frame: Message) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        channel.send((frame, tx)).await.unwrap();
        rx.await.unwrap()
    }
    pub async fn send_raw(&self, packet: RawPacket) -> Result<()> {
        self.send_frame(Message::binary(packet)).await
    }
    async fn tx_task(
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
                _ => break
            }
        }
    }
    async fn heart_beat_task(
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
                Either::Right(_) => {
                    println!("heart_beat_task kill due to signal");
                    break;
                }
            }
        }
    }
    async fn rx_task(
        callback: Arc<dyn Fn(Packet) + Send + Sync>,
        mut rx: RxType,
        mut kill: broadcast::Receiver<()>,
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
                                        (*callback)(Packet::from(raw))
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
                Either::Right(_) => {
                    println!("rx_task kill due to signal");
                    break;
                }
                _ => {
                    println!("rx_task kill due to unexpectedly dropped channel");
                    break;
                }
            }
        }
        Ok(())
    }
    async fn enter_room(&self) -> Result<()> {
        // "protover": 2,
        let protover = if self.compression { 2 } else { 1 };
        let req = json!({
          "clientver": "1.6.3",
          "platform": "web",
          "protover": protover,
          "roomid": self.room_id,
          "uid": self.uid,
          "type": 2
        });

        // TODO buffer proto
        let pack = RawPacket::new(
            Operation::RoomEnter,
            Protocol::Json,
            serde_json::to_vec(&req).unwrap(),
        );
        println!("sending room enter package");
        self.send_raw(pack).await?;
        println!("room enter package sent");
        Ok(())
    }
}
