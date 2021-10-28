/// The configuration for bilibili live stream connection.
#[derive(Debug, Clone)]
pub struct Stream(Box<StreamInner>);

impl Stream {
    pub fn new(room_id: u64, uid: u64, token: String, servers: Vec<String>) -> Self {
        Self(Box::new(StreamInner {
            room_id,
            uid,
            token,
            servers,
        }))
    }
}

impl Stream {
    pub const fn room_id(&self) -> u64 {
        self.0.room_id
    }
    pub const fn uid(&self) -> u64 {
        self.0.uid
    }
    pub fn token(&self) -> &str {
        &self.0.token
    }
    pub fn servers(&self) -> &[String] {
        &self.0.servers
    }
}

#[derive(Debug, Clone)]
struct StreamInner {
    /// Live room id (long version).
    room_id: u64,
    /// Live room user id.
    uid: u64,
    /// Danmaku server token.
    token: String,
    /// Danmaku server urls.
    servers: Vec<String>,
}
