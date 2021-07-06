use std::convert::TryInto;

use serde::Deserialize;

pub use types::*;

use crate::errors::Result;
use self::raw::RawPacket;
use crate::errors::ParseError;

mod parser;
pub mod raw;
mod types;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Packet {
    op: Operation,
    proto: Protocol,
    data: Vec<u8>,
}

impl Packet {
    pub fn new<T: Into<Vec<u8>>>(op: Operation, proto: Protocol, data: T) -> Self {
        Packet {
            op,
            proto,
            data: data.into(),
        }
    }
    pub fn op(&self) -> Operation {
        self.op
    }
    pub fn proto(&self) -> Protocol {
        self.proto
    }
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }
    pub fn json<'a, T: Deserialize<'a>>(&'a self) -> Result<T> {
        serde_json::from_slice(&self.data).map_err(|e| ParseError::JSON(e).into())
    }
    pub fn int32_be(&self) -> Result<i32> {
        Ok(i32::from_be_bytes(
            self.data
                .as_slice()
                .try_into()
                .map_err(|_| ParseError::Int32BE)?,
        ))
    }
}

impl From<RawPacket> for Packet {
    fn from(pack: RawPacket) -> Self {
        Packet {
            op: pack.op,
            proto: pack.protocol_version,
            data: pack.data,
        }
    }
}

impl From<Packet> for RawPacket {
    fn from(pack: Packet) -> Self {
        RawPacket::new(pack.op, pack.proto, pack.data)
    }
}
