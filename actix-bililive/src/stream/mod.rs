//! Bilibili live stream.

use bytes::Bytes;

pub use codec::Codec;
pub use pingpong::PingPongStream;

use crate::core::packet::Packet;

mod codec;
mod pingpong;
#[cfg(test)]
mod tests;

/// Either a valid bililive packet or a websocket ping message.
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
