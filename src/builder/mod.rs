use crate::builder::http::HTTPClient;
use crate::builder::types::{ConfQueryInner, Resp, RoomQueryInner};
use crate::config::StreamConfig;
use crate::errors::{BililiveError, ParseError, Result};

mod http;
#[cfg(test)]
pub(crate) mod tests;
mod types;

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

    pub async fn fetch_conf(mut self) -> Result<Self> {
        let resp: Resp<ConfQueryInner> = self
            .http
            .get_json("https://api.live.bilibili.com/room/v1/Danmu/getConf")
            .await?;

        self.token = Some(resp.token().to_string());
        self.servers = Some(resp.servers());
        Ok(self)
    }

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
