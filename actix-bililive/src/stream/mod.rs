use bytes::Bytes;

use crate::core::packet::Packet;

mod codec;
mod pingpong;

#[derive(Debug)]
pub enum PacketOrPing {
    Packet(Packet),
    PingPong(Bytes),
}

impl From<Packet> for PacketOrPing {
    fn from(pack: Packet) -> Self {
        Self::Packet(pack)
    }
}
