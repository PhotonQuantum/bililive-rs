use crate::ClientBuilder;

use super::types::{Resp, RoomQueryInner};

#[test]
fn must_parse_room_id() {
    let data = r#"{"code":0,"msg":"","message":"","data":{"status":0,"url":"https://live.bilibili.com/1016"}}"#;
    let parsed: Resp<RoomQueryInner> =
        serde_json::from_str(data).expect("unable to parse response");
    assert_eq!(parsed.room_id().expect("unable to get room id"), 1016);
}

#[tokio::test]
async fn must_build_client() {
    let callback = |pack| {};
    ClientBuilder::new()
        .room_id(1016)
        .callback(Box::new(callback))
        .build()
        .expect("unable to build client");
}
