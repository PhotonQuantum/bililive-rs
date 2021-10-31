//! Packet types.

use std::convert::TryInto;
use std::io::{Cursor, Read, Write};

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use nom::Err;
use serde::Deserialize;
use serde_json::json;

pub use types::*;

use crate::config::StreamConfig;
use crate::errors::{IncompleteResult, ParseError};

mod parser;
mod types;

#[cfg(test)]
mod tests;

type Result<T, E = ParseError> = std::result::Result<T, E>;

/// Bililive packet.
///
/// Packet can be used to encode/parse raw bilibili live packets, and extract information from it.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Packet {
    packet_length: u32,
    header_length: u16,
    protocol_version: Protocol,
    op: Operation,
    seq_id: u32,
    data: Vec<u8>,
}

impl Packet {
    /// Set the protocol version.
    pub fn set_proto(&mut self, protocol_version: Protocol) {
        self.protocol_version = protocol_version;
    }
    /// Set the operation.
    pub fn set_op(&mut self, op: Operation) {
        self.op = op;
    }
    /// Set the sequence id. By default it's 1.
    pub fn set_seq_id(&mut self, seq_id: u32) {
        self.seq_id = seq_id;
    }
    /// Set the packet body.
    /// Packet length will be updated automatically.
    pub fn set_data<T: Into<Vec<u8>>>(&mut self, data: T) {
        self.data = data.into();
        self.packet_length = self.header_length as u32 + self.data.len() as u32;
    }
}

impl Packet {
    /// Construct a new packet.
    ///
    /// To construct a zlib-compressed packet, you should create a JSON/Int32BE packet first,
    /// then call [`Packet::compress`](Packet::compress) to convert it to a zlib one.
    pub fn new<T: Into<Vec<u8>>>(op: Operation, protocol_version: Protocol, data: T) -> Self {
        let data = data.into();

        Self {
            packet_length: data.len() as u32 + 16,
            header_length: 16,
            protocol_version,
            op,
            seq_id: 1,
            data,
        }
    }

    /// Convert a JSON/Int32BE packet to a zlib-compressed one.
    ///
    /// # Errors
    /// Return errors if compression fails.
    pub fn compress(self) -> Result<Self> {
        let raw = self.encode();

        let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
        z.write_all(&raw)?;
        let data = z.finish()?;

        Ok(Self::new(self.op, Protocol::Zlib, data))
    }
}

impl Packet {
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn new_room_enter(config: &StreamConfig) -> Self {
        Self::new(
            Operation::RoomEnter,
            Protocol::Json,
            serde_json::to_vec(&json!({
                "uid": config.uid(),
                "roomid": config.room_id(),
                "protover": 2,
                "platform": "web",
                "clientver": "1.8.2",
                "type": 2,
                "key": config.token()
            }))
            .unwrap(),
        )
    }
}

impl Packet {
    /// Get the packet length.
    #[must_use]
    pub const fn packet_length(&self) -> u32 {
        self.packet_length
    }
    /// Get the header length.
    #[must_use]
    pub const fn header_length(&self) -> u16 {
        self.header_length
    }
    /// Get the sequence id.
    #[must_use]
    pub const fn seq_id(&self) -> u32 {
        self.seq_id
    }
    /// Get the operation.
    #[must_use]
    pub const fn op(&self) -> Operation {
        self.op
    }
    /// Get the protocol version.
    #[must_use]
    pub const fn proto(&self) -> Protocol {
        self.protocol_version
    }
    /// Get bytes of the body.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }
    /// Try to parse the body by json.
    ///
    /// # Errors
    /// It may fail if the model is incorrect or it's not a json packet.
    /// You may check the type of the packet by [`Packet::proto`](Packet::proto).
    pub fn json<'a, T: Deserialize<'a>>(&'a self) -> Result<T> {
        serde_json::from_slice(&self.data).map_err(ParseError::Json)
    }
    /// Try to parse the body by big endian int32.
    ///
    /// # Errors
    /// It may fail if it's not a int packet.
    /// You may check the type of the packet by [`Packet::proto`](Packet::proto).
    pub fn int32_be(&self) -> Result<i32> {
        Ok(i32::from_be_bytes(
            self.data
                .as_slice()
                .try_into()
                .map_err(|_| ParseError::Int32BE)?,
        ))
    }
}

impl Packet {
    /// Encode the packet into bytes ready to be sent to the server.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.packet_length as usize);
        buf.extend(self.packet_length.to_be_bytes());
        buf.extend(self.header_length.to_be_bytes());
        buf.extend((self.protocol_version as u16).to_be_bytes());
        buf.extend((self.op as u32).to_be_bytes());
        buf.extend(self.seq_id.to_be_bytes());
        buf.extend(&self.data);
        buf
    }

    /// Parse the packet received from Bilibili server.
    #[must_use]
    pub fn parse(input: &[u8]) -> IncompleteResult<(&[u8], Self)> {
        match parser::parse(input) {
            Ok((input, packet)) => {
                if let Protocol::Zlib = packet.protocol_version {
                    let mut z = ZlibDecoder::new(Cursor::new(packet.data));
                    let mut buf = Vec::new();
                    if let Err(e) = z.read_to_end(&mut buf) {
                        return IncompleteResult::Err(ParseError::ZlibError(e));
                    }

                    match parser::parse(&buf) {
                        Ok((_, packet)) => IncompleteResult::Ok((input, packet)),
                        Err(Err::Incomplete(needed)) => {
                            IncompleteResult::Err(ParseError::PacketError(format!(
                                "incomplete buffer: {:?} needed",
                                needed
                            )))
                        }
                        Err(Err::Error(e) | Err::Failure(e)) => {
                            IncompleteResult::Err(ParseError::PacketError(format!("{:?}", e)))
                        }
                    }
                } else {
                    IncompleteResult::Ok((input, packet))
                }
            }
            Err(Err::Incomplete(needed)) => IncompleteResult::Incomplete(needed),
            Err(Err::Error(e) | Err::Failure(e)) => {
                IncompleteResult::Err(ParseError::PacketError(format!("{:?}", e)))
            }
        }
    }
}
