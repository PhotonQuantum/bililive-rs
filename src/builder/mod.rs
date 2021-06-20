use std::sync::Arc;

use crate::builder::types::{ConfQueryInner, Resp, RoomQueryInner};
use crate::errors::{BililiveError, ParseError, Result};
use crate::{Client, Packet};

#[cfg(test)]
mod tests;
mod types;

pub struct ClientBuilder {
    http: reqwest::Client,
    compression: bool,
    room_id: Option<u64>,
    uid: Option<u64>,
    token: Option<String>,
    servers: Option<Vec<String>>,
    tx_buffer: usize,
    callback: Option<Box<dyn Fn(Packet) + Send + Sync>>,
}

impl Default for ClientBuilder {
    #[must_use]
    fn default() -> Self {
        Self::new_with_http(Default::default())
    }
}

impl ClientBuilder {
    #[must_use]
    pub fn new() -> Self {
        Default::default()
    }

    #[must_use]
    pub fn new_with_http(http: reqwest::Client) -> Self {
        Self {
            http,
            compression: false,
            room_id: None,
            uid: None,
            token: None,
            servers: None,
            tx_buffer: 32,
            callback: None,
        }
    }

    setter_copy!(compression, bool);
    setter_copy!(tx_buffer, usize);
    setter_option_copy!(room_id, u64);
    setter_option_copy!(uid, u64);
    setter_option_copy!(callback, Box<dyn Fn(Packet) + Send + Sync>);
    setter_option_clone!(token, String);
    setter_option_clone!(servers, Vec<String>);

    pub async fn by_uid(mut self, uid: u64) -> Result<Self> {
        let data = self
            .http
            .get(format!(
                "https://api.live.bilibili.com/bili/living_v2/{}",
                uid
            ))
            .send()
            .await?
            .bytes()
            .await?;
        let resp: Resp<RoomQueryInner> = serde_json::from_slice(&data).map_err(ParseError::JSON)?;
        let room_id = resp.room_id().ok_or(ParseError::RoomId)?;

        self.room_id = Some(room_id);
        self.uid = Some(uid);
        Ok(self)
    }

    pub async fn fetch_conf(mut self) -> Result<Self> {
        let data = self
            .http
            .get("https://api.live.bilibili.com/room/v1/Danmu/getConf")
            .send()
            .await?
            .bytes()
            .await?;
        let resp: Resp<ConfQueryInner> = serde_json::from_slice(&data).map_err(ParseError::JSON)?;

        self.token = Some(resp.token().to_string());
        self.servers = Some(resp.servers());
        Ok(self)
    }

    pub fn build(self) -> Result<Client> {
        Ok(Client::new(
            self.room_id
                .ok_or_else(|| BililiveError::Build(String::from("room_id")))?,
            self.uid
                .ok_or_else(|| BililiveError::Build(String::from("uid")))?,
            self.token
                .ok_or_else(|| BililiveError::Build(String::from("token")))?,
            self.servers
                .ok_or_else(|| BililiveError::Build(String::from("servers")))?,
            self.compression,
            self.tx_buffer,
            self.callback
                .map(Arc::from)
                .ok_or_else(|| BililiveError::Build(String::from("callback")))?,
        ))
    }
}
