use bytes::Bytes;

pub use codec::Codec;
pub use pingpong::PingPongStream;

use crate::core::packet::Packet;

mod codec;
mod pingpong;
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
