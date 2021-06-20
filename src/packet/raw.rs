use std::io::{Cursor, Read};

use flate2::read::ZlibDecoder;
use nom::Err;

use crate::{IncompleteResult, ParseError};

use super::types::{Operation, Protocol};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RawPacket {
    pub packet_length: u32,
    pub header_length: u16,
    pub protocol_version: Protocol,
    pub op: Operation,
    pub seq_id: u32,
    pub data: Vec<u8>,
}

impl RawPacket {
    pub fn parse(input: &[u8]) -> IncompleteResult<(&[u8], RawPacket)> {
        match super::parser::parse(input) {
            Ok((input, packet)) => {
                if let Protocol::Buffer = packet.protocol_version {
                    let mut z = ZlibDecoder::new(Cursor::new(packet.data));
                    let mut buf = Vec::new();
                    if let Err(e) = z.read_to_end(&mut buf) {
                        return IncompleteResult::Err(ParseError::ZlibError(e).into());
                    }

                    match super::parser::parse(&buf) {
                        Ok((_, packet)) => IncompleteResult::Ok((input, packet)),
                        Err(Err::Incomplete(needed)) => IncompleteResult::Err(
                            ParseError::PacketError(format!(
                                "incomplete buffer: {:?} needed",
                                needed
                            ))
                            .into(),
                        ),
                        Err(Err::Error(e) | Err::Failure(e)) => IncompleteResult::Err(
                            ParseError::PacketError(format!("{:?}", e)).into(),
                        ),
                    }
                } else {
                    IncompleteResult::Ok((input, packet))
                }
            }
            Err(Err::Incomplete(needed)) => IncompleteResult::Incomplete(needed),
            Err(Err::Error(e) | Err::Failure(e)) => {
                IncompleteResult::Err(ParseError::PacketError(format!("{:?}", e)).into())
            }
        }
    }

    pub fn new<T: Into<Vec<u8>>>(op: Operation, proto: Protocol, data: T) -> Self {
        let data = data.into();
        if let Protocol::Buffer = proto {
            unimplemented!();
        }

        Self {
            packet_length: data.len() as u32 + 16,
            header_length: 16,
            protocol_version: proto,
            op,
            seq_id: 1,
            data,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.packet_length as usize + self.header_length as usize);
        buf.extend(self.packet_length.to_be_bytes());
        buf.extend(self.header_length.to_be_bytes());
        buf.extend((self.protocol_version as u16).to_be_bytes());
        buf.extend((self.op as u32).to_be_bytes());
        buf.extend(self.seq_id.to_be_bytes());
        buf.extend(&self.data);
        buf
    }
}

impl From<&RawPacket> for Vec<u8> {
    fn from(pack: &RawPacket) -> Self {
        pack.to_bytes()
    }
}

impl From<RawPacket> for Vec<u8> {
    fn from(pack: RawPacket) -> Self {
        pack.to_bytes()
    }
}
