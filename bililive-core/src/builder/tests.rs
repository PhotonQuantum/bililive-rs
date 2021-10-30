use crate::builder::ConfigBuilder;

use super::types::{ConfQueryInner, Resp, RoomQueryInner};

#[test]
fn must_parse_room_id() {
    let data = r#"{"code":0,"msg":"","message":"","data":{"status":0,"url":"https://live.bilibili.com/1016"}}"#;
    let parsed: Resp<RoomQueryInner> =
        serde_json::from_str(data).expect("unable to parse response");
    assert_eq!(parsed.room_id(), 1016);
}

#[test]
fn must_parse_conf() {
    let data = include_str!("../../tests/getConf.json");
    let parsed: Resp<ConfQueryInner> =
        serde_json::from_str(data).expect("unable to parse response");
    assert_eq!(
        parsed.token(),
        "zRLe_Wb0lwdalke2_OMvIxBD7uBQ7pNKepn-fP2rIV91AyCRSAYwsw1CVYGgjtuf8IA1AHLchDXhiekQ3IMWnzBu5zqIK9CqdY-tuaCpVi1fxE_hqBEdsfdgxPJyFQAxtgqK4cdf1dm7"
    );
    assert_eq!(
        parsed.servers(),
        [
            "wss://tx-gz-live-comet-03.chat.bilibili.com:443/sub",
            "wss://tx-sh-live-comet-03.chat.bilibili.com:443/sub",
            "wss://broadcastlv.chat.bilibili.com:443/sub"
        ]
    )
}

#[test]
fn must_build_config() {
    ConfigBuilder::<(), _, _, _, _>::new()
        .room_id(1016)
        .uid(0)
        .servers(&["wss://".to_string()])
        .token("asdf")
        .build();
}
