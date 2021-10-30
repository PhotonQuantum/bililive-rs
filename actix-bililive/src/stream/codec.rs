use actix_codec::{Decoder, Encoder};
use awc::error::WsClientError;
use awc::ws::Codec as WsCodec;
use awc::ws::{Frame, Message};
use bytes::BytesMut;
use log::{debug, warn};

use crate::core::errors::{IncompleteResult, Stream};
use crate::core::packet::Packet;

use super::PacketOrPing;

#[derive(Debug)]
pub struct Codec {
    ws_codec: WsCodec,
    read_buffer: Vec<u8>,
}

impl Codec {
    #[must_use]
    pub const fn new(ws_codec: WsCodec) -> Self {
        Self {
            ws_codec,
            read_buffer: vec![],
        }
    }
}

impl Decoder for Codec {
    type Item = PacketOrPing;
    type Error = Stream<WsClientError>;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let ws_frame = if let Some(frame) = self
            .ws_codec
            .decode(src)
            .map_err(|e| Stream::from_ws_error(e.into()))?
        {
            frame
        } else {
            return Ok(None);
        };

        match ws_frame {
            Frame::Binary(bytes) => {
                self.read_buffer.extend_from_slice(&bytes);

                match Packet::parse(&self.read_buffer) {
                    IncompleteResult::Ok((remaining, pack)) => {
                        debug!("packet parsed, {} bytes remaining", remaining.len());

                        // remove parsed bytes
                        let consume_len = self.read_buffer.len() - remaining.len();
                        drop(self.read_buffer.drain(..consume_len));

                        Ok(Some(pack.into()))
                    }
                    IncompleteResult::Incomplete(needed) => {
                        debug!("incomplete packet, {:?} needed", needed);
                        Ok(None)
                    }
                    IncompleteResult::Err(e) => {
                        warn!("error occurred when parsing incoming packet");
                        Err(e.into())
                    }
                }
            }
            Frame::Ping(bytes) => {
                debug!("incoming ws ping");
                Ok(Some(PacketOrPing::PingPong(bytes)))
            }
            _ => {
                debug!("not a binary message, dropping");
                Ok(None)
            }
        }
    }
}

impl Encoder<PacketOrPing> for Codec {
    type Error = Stream<WsClientError>;

    fn encode(&mut self, item: PacketOrPing, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = match item {
            PacketOrPing::Packet(pack) => Message::Binary(pack.encode().into()),
            PacketOrPing::PingPong(bytes) => Message::Pong(bytes),
        };
        self.ws_codec
            .encode(msg, dst)
            .map_err(|e| Stream::from_ws_error(e.into()))
    }
}
