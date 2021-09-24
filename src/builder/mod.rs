//! Bililive config builders

use crate::builder::http::HTTPClient;
use crate::builder::types::{ConfQueryInner, Resp, RoomQueryInner};
use crate::config::StreamConfig;
use crate::errors::{BililiveError, ParseError, Result};

mod http;
#[cfg(test)]
pub(crate) mod tests;
mod types;

/// Bililive stream config builder
///
/// Stream config can be built via given live room parameters (room id and user id) & danmaku server configs (server token and list).
///
/// # Helper methods
///
/// [`by_uid`](ConfigBuilder::by_uid) fetches room id by given user id.
///
/// [`fetch_conf`](ConfigBuilder::fetch_conf) fetches danmaku server token and list without any input parameter.
///
/// # Example
///
/// ```rust
/// # use bililive::{ConfigBuilder, BililiveError, StreamConfig};
/// #
/// # let fut = async {
/// ConfigBuilder::new()
///     .by_uid(1472906636)
///     .await?
///     .fetch_conf()
///     .await?
///     .build()
/// # };
/// #
/// # tokio_test::block_on(fut).unwrap();
/// ```
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    http: HTTPClient,
    room_id: Option<u64>,
    uid: Option<u64>,
    token: Option<String>,
    servers: Option<Vec<String>>,
}

impl ConfigBuilder {
    #[must_use]
    pub fn new() -> Self {
        Default::default()
    }

    setter_option_copy!(room_id, u64);
    setter_option_copy!(uid, u64);

    #[must_use]
    pub fn token(mut self, token: &str) -> Self {
        self.token = Some(token.to_string());
        self
    }

    #[must_use]
    pub fn servers(mut self, servers: &[String]) -> Self {
        self.servers = Some(servers.to_vec());
        self
    }

    /// Fills `room_id` and `uid` by given `uid`, fetching `room_id` automatically.
    ///
    /// # Errors
    /// Returns an error when HTTP api request fails.
    pub async fn by_uid(mut self, uid: u64) -> Result<Self> {
        let resp: Resp<RoomQueryInner> = self
            .http
            .get_json(&*format!(
                "https://api.live.bilibili.com/bili/living_v2/{}",
                uid
            ))
            .await?;
        let room_id = resp.room_id().ok_or(ParseError::RoomId)?;

        self.room_id = Some(room_id);
        self.uid = Some(uid);
        Ok(self)
    }

    /// Fetches danmaku server configs & uris
    ///
    /// # Errors
    /// Returns an error when HTTP api request fails.
    pub async fn fetch_conf(mut self) -> Result<Self> {
        let resp: Resp<ConfQueryInner> = self
            .http
            .get_json("https://api.live.bilibili.com/room/v1/Danmu/getConf")
            .await?;

        self.token = Some(resp.token().to_string());
        self.servers = Some(resp.servers());
        Ok(self)
    }

    /// Consumes the builder and returns [`StreamConfig`](StreamConfig)
    ///
    /// # Errors
    /// Returns an error when there's field missing.
    pub fn build(self) -> Result<StreamConfig> {
        Ok(StreamConfig {
            room_id: self
                .room_id
                .ok_or_else(|| BililiveError::Build(String::from("room_id")))?,
            uid: self
                .uid
                .ok_or_else(|| BililiveError::Build(String::from("uid")))?,
            token: self
                .token
                .ok_or_else(|| BililiveError::Build(String::from("token")))?,
            servers: self
                .servers
                .ok_or_else(|| BililiveError::Build(String::from("servers")))?,
        })
    }
}
