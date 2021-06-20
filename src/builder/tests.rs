use crate::ClientBuilder;

use super::types::{ConfQueryInner, Resp, RoomQueryInner};

#[test]
fn must_parse_room_id() {
    let data = r#"{"code":0,"msg":"","message":"","data":{"status":0,"url":"https://live.bilibili.com/1016"}}"#;
    let parsed: Resp<RoomQueryInner> =
        serde_json::from_str(data).expect("unable to parse response");
    assert_eq!(parsed.room_id().expect("unable to get room id"), 1016);
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

#[tokio::test]
async fn must_build_client() {
    let callback = |_pack| {};
    ClientBuilder::new()
        .room_id(1016)
        .uid(0)
        .servers(&["wss://".to_string()])
        .token("asdf")
        .callback(Box::new(callback))
        .build()
        .expect("unable to build client");
}

#[tokio::test]
async fn must_build_client_real() {
    let callback = |_pack| {};
    ClientBuilder::new()
        .by_uid(419220)
        .await?
        .callback(Box::new(callback))
        .fetch_conf()
        .await?
        .build()
        .expect("unable to build client");
}