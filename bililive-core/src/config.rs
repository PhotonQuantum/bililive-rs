//! Configuration types.

/// The configuration for bilibili live stream connection.
#[derive(Debug, Clone)]
pub struct StreamConfig(Box<StreamConfigInner>);

impl StreamConfig {
    #[must_use]
    pub fn new(room_id: u64, uid: u64, token: String, servers: Vec<String>) -> Self {
        Self(Box::new(StreamConfigInner {
            room_id,
            uid,
            token,
            servers,
        }))
    }
}

impl StreamConfig {
    /// Live room id (long version).
    #[must_use]
    pub const fn room_id(&self) -> u64 {
        self.0.room_id
    }
    /// Live room user id.
    #[must_use]
    pub const fn uid(&self) -> u64 {
        self.0.uid
    }
    /// Danmaku server token.
    #[must_use]
    pub fn token(&self) -> &str {
        &self.0.token
    }
    /// Danmaku server urls.
    #[must_use]
    pub fn servers(&self) -> &[String] {
        &self.0.servers
    }
}

#[derive(Debug, Clone)]
struct StreamConfigInner {
    /// Live room id (long version).
    room_id: u64,
    /// Live room user id.
    uid: u64,
    /// Danmaku server token.
    token: String,
    servers: Vec<String>,
}
