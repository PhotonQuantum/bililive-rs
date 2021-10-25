use std::fs::read;

use serde_json::json;

use crate::errors::IncompleteResult;
use crate::{Operation, Packet, Protocol};

fn test_packet(path: &str, expect: Packet, skip_encode: bool) {
    let content = read(path).unwrap();
    let res = Packet::parse(&content);
    if let IncompleteResult::Ok((_, packet)) = res {
        assert_eq!(packet, expect);
        if !skip_encode {
            assert_eq!(content, expect.encode());
        }
    } else {
        panic!("error while parsing");
    }
}

#[test]
fn must_parse_int32be() {
    // This package is a bit strange ..
    let mut expected = Packet::new(
        Operation::HeartBeatResponse,
        Protocol::Json,
        358069i32.to_be_bytes(),
    );
    expected.set_seq_id(0);
    test_packet("tests/raw/int32be.packet", expected, false);
}

#[test]
fn must_parse_json() {
    test_packet(
        "tests/raw/json.packet",
        Packet::new(
            Operation::RoomEnterResponse,
            Protocol::Json,
            serde_json::to_vec(&json!({
            "code": 0
            }))
            .unwrap(),
        ),
        false,
    );
}

#[test]
fn must_parse_buffer() {
    let json_value = json!({
        "cmd": "INTERACT_WORD",
        "data": {
        "contribution": {"grade": 0},
        "dmscore": 2,
        "fans_medal": {
            "anchor_roomid": 0,
            "guard_level": 0,
            "icon_id": 0,
            "is_lighted": 0,
            "medal_color": 0,
            "medal_color_border": 0,
            "medal_color_end": 0,
            "medal_color_start": 0,
            "medal_level": 0,
            "medal_name": "",
            "score": 0,
            "special": "",
            "target_id": 0
        },
        "identities": [1],
        "is_spread": 0,
        "msg_type": 1,
        "roomid": 23090051,
        "score": 1626324624442i64,
        "spread_desc": "",
        "spread_info": "",
        "tail_icon": 0,
        "timestamp": 1626324624,
        "trigger_time": 1626324623404263200i64,
        "uid": 174102117,
        "uname": "vioIet・伊芙加登",
        "uname_color": ""
    }});
    let mut expected = Packet::new(
        Operation::Notification,
        Protocol::Json,
        serde_json::to_vec(&json_value).unwrap(),
    );
    expected.set_seq_id(0);
    test_packet("tests/raw/buffer.packet", expected, true);
}
