use std::borrow::BorrowMut;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::errors::Result;
use crate::packet::Packet;

mod actions;
mod send;
mod tasks;

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
    room_id: u64,
    uid: u64,
    token: String,
    servers: Vec<String>,
    compression: bool,
    tx_buffer: usize,
    popularity: Arc<AtomicI32>,
    callback: Arc<dyn Fn(Packet) + Send + Sync>,
    handles: Option<JoinHandles>,
    tx_channel: Option<mpsc::Sender<ChannelType>>,
    kill: broadcast::Sender<()>,
}

// TODO reconnect after unexpectedly disconnected
impl Client {
    pub(crate) fn new(
        room_id: u64,
        uid: u64,
        token: String,
        servers: Vec<String>,
        compression: bool,
        tx_buffer: usize,
        callback: Arc<dyn Fn(Packet) + Send + Sync>,
    ) -> Self {
        let (kill, _) = broadcast::channel(32);
        Self {
            room_id,
            uid,
            token,
            servers,
            compression,
            tx_buffer,
            popularity: Arc::new(Default::default()),
            callback,
            handles: None,
            tx_channel: None,
            kill,
        }
    }

    pub fn room_id(&self) -> u64 {
        self.room_id
    }

    pub fn uid(&self) -> u64 {
        self.uid
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn servers(&self) -> Vec<&str> {
        self.servers.iter().map(|s| s.as_str()).collect::<Vec<_>>()
    }

    pub fn popularity(&self) -> i32 {
        self.popularity.load(Relaxed)
    }

    pub fn compression(&self) -> bool {
        self.compression
    }

    pub async fn connect(&mut self) -> Result<()> {
        // TODO try servers
        let (stream, _) = connect_async(self.servers.first().unwrap()).await?;
        let (tx, rx) = stream.split();

        let (tx_chan, rx_chan) = mpsc::channel(self.tx_buffer);

        self.tx_channel = Some(tx_chan.clone());

        println!("spawning tasks");
        let tx_handle = tokio::spawn(Self::tx_task(tx, rx_chan, self.kill.subscribe()));
        let rx_handle = tokio::spawn(Self::rx_task(
            self.callback.clone(),
            rx,
            self.kill.subscribe(),
            self.popularity.clone(),
        ));

        self.enter_room().await?;

        let heartbeat_handle = tokio::spawn(Self::heart_beat_task(tx_chan, self.kill.subscribe()));
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
}
