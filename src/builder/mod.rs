#[cfg(test)]
mod tests;
mod types;

use crate::builder::types::RoomQueryResponse;
use crate::errors::{BililiveError, ParseError, Result};
use crate::Client;

pub struct ClientBuilder {
    http: reqwest::Client,
    compression: bool,
    room_id: Option<u64>,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new_with_http(Default::default())
    }
}

impl ClientBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_with_http(http: reqwest::Client) -> Self {
        Self {
            http,
            compression: false,
            room_id: None,
        }
    }

    setter_copy!(compression, bool);
    setter_option_copy!(room_id, u64);

    pub async fn room_id_by_uid(mut self, uid: u64) -> Result<Self> {
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
        let resp: RoomQueryResponse = serde_json::from_slice(&data).map_err(ParseError::JSON)?;
        let room_id = resp.room_id().ok_or(ParseError::RoomId)?;

        self.room_id = Some(room_id);
        Ok(self)
    }

    pub fn build(self) -> Result<Client> {
        Ok(Client {
            http: self.http,
            room_id: self
                .room_id
                .ok_or_else(|| BililiveError::BuildError(String::from("room_id")))?,
        })
    }
}

// https://api.live.bilibili.com/bili/living_v2/419220
