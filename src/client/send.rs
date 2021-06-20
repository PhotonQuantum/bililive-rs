use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;

use crate::errors::{BililiveError, Result};
use crate::packet::raw::RawPacket;
use crate::Client;

use super::ChannelType;

impl Client {
    pub(crate) async fn send_frame(&self, frame: Message) -> Result<()> {
        let channel = self
            .tx_channel
            .as_ref()
            .ok_or(BililiveError::NotConnected)?;
        Self::_send_frame(channel, frame).await
    }
    pub(crate) async fn _send_frame(
        channel: &mpsc::Sender<ChannelType>,
        frame: Message,
    ) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        channel.send((frame, tx)).await.unwrap();
        rx.await.unwrap()
    }
    pub async fn send_raw(&self, packet: RawPacket) -> Result<()> {
        self.send_frame(Message::binary(packet)).await
    }
}
