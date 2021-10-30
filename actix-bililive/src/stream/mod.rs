use bytes::Bytes;

use crate::core::packet::Packet;

pub mod codec;
pub mod pingpong;
#[cfg(test)]
mod tests;

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
