use async_tungstenite::tungstenite::Message;
use serde_json::json;

use crate::config::StreamConfig;
use bililive_core::packet::{Operation, Protocol, Packet};

pub fn room_enter_message(config: &StreamConfig) -> Message {
    Message::binary(
        Packet::new(
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
        .encode(),
    )
}
