use serde_json::json;
use tokio_tungstenite::tungstenite::Message;

use crate::{Operation, Protocol};
use crate::config::StreamConfig;
use crate::raw::RawPacket;

pub fn room_enter_message(config: &StreamConfig) -> Message {
    Message::binary(RawPacket::new(
        Operation::RoomEnter,
        Protocol::Json,
        serde_json::to_vec(&json!({
            "uid": config.uid,
            "roomid": config.room_id,
            "protover": 2,
            "platform": "web",
            "clientver": "1.8.2",
            "type": 2,
            "key": config.token
        }))
        .unwrap(),
    ))
}
