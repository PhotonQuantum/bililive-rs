use std::convert::TryFrom;

use crate::errors::{Parse as ParseError};

/// Live event types.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u32)]
pub enum Operation {
    HeartBeat = 2,
    HeartBeatResponse = 3,
    Notification = 5,
    RoomEnter = 7,
    RoomEnterResponse = 8,
    Unknown = u32::MAX,
}

impl From<u32> for Operation {
    fn from(i: u32) -> Self {
        match i {
            2 => Self::HeartBeat,
            3 => Self::HeartBeatResponse,
            5 => Self::Notification,
            7 => Self::RoomEnter,
            8 => Self::RoomEnterResponse,
            _ => Self::Unknown,
        }
    }
}

/// Protocol types.
///
/// Indicating the format of packet content.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u16)]
pub enum Protocol {
    Json = 0,
    Int32BE = 1,
    Zlib = 2,
}

impl TryFrom<u16> for Protocol {
    type Error = ParseError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Json),
            1 => Ok(Self::Int32BE),
            2 => Ok(Self::Zlib),
            _ => Err(ParseError::UnknownProtocol),
        }
    }
}
