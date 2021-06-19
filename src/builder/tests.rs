use super::types::RoomQueryResponse;

#[test]
fn must_parse_room_id() {
    let data = r#"{"code":0,"msg":"","message":"","data":{"status":0,"url":"https://live.bilibili.com/1016"}}"#;
    let parsed: RoomQueryResponse = serde_json::from_str(data).expect("unable to parse response");
    assert_eq!(parsed.room_id().expect("unable to get room id"), 1016);
}
