use bililive_core::config::Stream as StreamConfig;

use crate::ConfigBuilder;

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

#[test]
fn must_build_config() {
    ConfigBuilder::new()
        .room_id(1016)
        .uid(0)
        .servers(&["wss://".to_string()])
        .token("asdf")
        .build()
        .expect("unable to build client");
}

pub(crate) async fn build_real_config(override_servers: bool) -> StreamConfig {
    let builder = ConfigBuilder::new()
        .by_uid(419220)
        .await
        .expect("unable to fetch room_id")
        .fetch_conf()
        .await
        .expect("unable to fetch server conf");
    let builder = if override_servers {
        builder.servers(&["wss://broadcastlv.chat.bilibili.com/sub".to_string()])
    } else {
        builder
    };
    builder.build().expect("unable to build client")
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn must_build_real_config_tokio() {
    build_real_config(false).await;
}

#[cfg(feature = "async-std")]
#[async_std::test]
async fn must_build_real_config_async_std() {
    build_real_config(false).await;
}
