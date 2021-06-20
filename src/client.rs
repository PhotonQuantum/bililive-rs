use std::sync::Arc;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{Sink, SinkExt, StreamExt};
use nom::Err;
use serde_json::json;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::errors::{BililiveError, Result};
use crate::packet::{raw::RawPacket, Operation, Protocol};
use crate::{Packet, ParseError};

type TxType = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type RxType = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

pub struct Client {
    pub(crate) http: reqwest::Client,
    pub(crate) room_id: u64,
    pub(crate) uid: u64,
    pub(crate) compression: bool,
    pub(crate) tx: Option<Arc<Mutex<TxType>>>,
    // pub(crate) rx: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    pub(crate) callback: Arc<dyn Fn(Packet) + Send + Sync>,
    pub(crate) handle: Option<(JoinHandle<Result<()>>, JoinHandle<()>)>,
}

impl Client {
    pub(crate) fn new(
        http: reqwest::Client,
        room_id: u64,
        uid: u64,
        compression: bool,
        callback: Arc<dyn Fn(Packet) + Send + Sync>,
    ) -> Self {
        Self {
            http,
            room_id,
            uid,
            compression,
            tx: None,
            callback,
            handle: None,
        }
    }
    pub fn room_id(&self) -> u64 {
        self.room_id
    }
    pub async fn connect(&mut self) -> Result<()> {
        let (stream, _) = connect_async("wss://broadcastlv.chat.bilibili.com/sub").await?;
        let (tx, rx) = stream.split();
        let tx = Arc::new(Mutex::new(tx));
        self.tx = Some(tx.clone());
        self.enter_room().await?;

        let main_handle = tokio::spawn(Self::event_loop(self.callback.clone(), rx));
        let heartbeat_handle = tokio::spawn(Self::heart_beat_loop(tx.clone()));
        self.handle = Some((main_handle, heartbeat_handle));

        Ok(())
    }
    pub async fn close(&mut self) -> Result<()> {
        self.tx
            .as_ref()
            .ok_or(BililiveError::NotConnected)?
            .lock()
            .await
            .send(Message::Close(None))
            .await?;

        Ok(())
    }
    pub async fn join(&mut self) -> Result<()> {
        if let Some((main_handle, heartbeat_handle)) = &mut self.handle {
            main_handle.await??;
            heartbeat_handle.await?;
        }

        Ok(())
    }
    async fn heart_beat_loop(tx: Arc<Mutex<TxType>>) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let mut tx = tx.lock().await;
            Self::send_heart_beat(&mut tx).await;
        }
    }
    async fn event_loop(callback: Arc<dyn Fn(Packet) + Send + Sync>, mut rx: RxType) -> Result<()> {
        let mut buf = vec![];
        while let Some(msg) = rx.next().await {
            match msg {
                Ok(msg) => {
                    if msg.is_binary() {
                        buf.extend(msg.into_data());
                        match RawPacket::parse(&buf) {
                            Ok((remaining, raw)) => {
                                println!("packet parsed, {} bytes remaining", remaining.len());
                                buf = remaining.to_vec(); // TODO to be optimized
                                (*callback)(Packet::from(raw))
                            }
                            Err(Err::Incomplete(needed)) => {
                                println!("incomplete packet, {:?} needed", needed);
                            }
                            Err(Err::Error(e) | Err::Failure(e)) => {
                                return Err(ParseError::PacketError(format!("{:?}", e)).into());
                            }
                        }
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }
    async fn enter_room(&self) -> Result<()> {
        let mut tx = self
            .tx
            .as_ref()
            .ok_or(BililiveError::NotConnected)?
            .lock()
            .await;

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
        tx.send(Message::binary(pack)).await?;
        println!("room enter package sent");
        Ok(())
    }
    async fn send_heart_beat(tx: &mut TxType) -> Result<()> {
        let pack = RawPacket::new(Operation::HeartBeat, Protocol::Json, vec![]);
        println!("sending heartbeat");
        tx.send(Message::binary(pack)).await?;
        println!("heartbeat sent");
        Ok(())
    }
}
