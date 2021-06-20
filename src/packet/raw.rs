use nom::IResult;

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
    pub fn parse(input: &[u8]) -> IResult<&[u8], RawPacket> {
        super::parser::parse(input)
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
