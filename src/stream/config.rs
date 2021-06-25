#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StreamConfig {
    // bilibili live room id (long)
    pub room_id: u64,
    // live user id
    pub uid: u64,
    // danmaku server token
    pub token: String,
    // danmaku server urls
    pub servers: Vec<String>,
}