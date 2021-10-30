//! `bililive` config builder.

use std::marker::PhantomData;

/// `bililive` stream config builder.
///
/// Stream config can be built via given live room parameters (room id and user id) & danmaku server configs (server token and list).
///
/// # Helper methods
///
/// [`by_uid`](ConfigBuilder::by_uid) fetches room id by given user id.
///
/// [`fetch_conf`](ConfigBuilder::fetch_conf) fetches danmaku server token and list without any input parameter.
///
/// See docs of downstream crates for details.
use async_trait::async_trait;
use serde::de::DeserializeOwned;

use crate::builder::types::{ConfQueryInner, Resp, RoomQueryInner};
use crate::config::StreamConfig;
use crate::errors::BuildError;

#[cfg(test)]
mod tests;
mod types;

type BoxedError = Box<dyn std::error::Error>;

/// An abstract HTTP client.
///
/// Used in [`ConfigBuilder`](ConfigBuilder) to help fetching bilibili config.
#[cfg(feature = "not-send")]
#[async_trait(? Send)]
pub trait Requester {
    /// Make a `GET` request to the url and try to deserialize the response body as JSON.
    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, BoxedError>;
}

/// An abstract HTTP client.
///
/// Used in [`ConfigBuilder`](ConfigBuilder) to help fetching bilibili config.
#[cfg(not(feature = "not-send"))]
#[async_trait]
pub trait Requester: Send + Sync {
    /// Make a `GET` request to the url and try to deserialize the response body as JSON.
    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, BoxedError>;
}

#[doc(hidden)]
pub enum BF {}

#[doc(hidden)]
pub enum BN {}

/// `bililive` stream config builder.
///
/// Stream config can be built via given live room parameters (room id and user id) & danmaku server configs (server token and list).
///
/// # Helper methods
///
/// [`by_uid`](ConfigBuilder::by_uid) fetches room id by given user id.
///
/// [`fetch_conf`](ConfigBuilder::fetch_conf) fetches danmaku server token and list without any input parameter.
#[derive(Debug)]
pub struct ConfigBuilder<H, R, U, T, S> {
    http: H,
    room_id: Option<u64>,
    uid: Option<u64>,
    token: Option<String>,
    servers: Option<Vec<String>>,
    __marker: PhantomData<(R, U, T, S)>,
}

impl<H: Default> ConfigBuilder<H, BN, BN, BN, BN> {
    /// Construct a new builder with default requester client.
    #[allow(clippy::new_without_default)]
    #[must_use]
    pub fn new() -> Self {
        Self::new_with_client(H::default())
    }
}

impl<H> ConfigBuilder<H, BN, BN, BN, BN> {
    /// Construct a new builder with given requester client.
    #[must_use]
    pub const fn new_with_client(client: H) -> Self {
        Self {
            http: client,
            room_id: None,
            uid: None,
            token: None,
            servers: None,
            __marker: PhantomData,
        }
    }
}

impl<H, R, U, T, S> ConfigBuilder<H, R, U, T, S> {
    #[allow(clippy::missing_const_for_fn)] // misreport
    fn cast<R2, U2, T2, S2>(self) -> ConfigBuilder<H, R2, U2, T2, S2> {
        ConfigBuilder {
            http: self.http,
            room_id: self.room_id,
            uid: self.uid,
            token: self.token,
            servers: self.servers,
            __marker: PhantomData,
        }
    }
}

impl<H, R, U, T, S> ConfigBuilder<H, R, U, T, S> {
    #[must_use]
    pub fn room_id(mut self, room_id: u64) -> ConfigBuilder<H, BF, U, T, S> {
        self.room_id = Some(room_id);
        self.cast()
    }
    #[must_use]
    pub fn uid(mut self, uid: u64) -> ConfigBuilder<H, R, BF, T, S> {
        self.uid = Some(uid);
        self.cast()
    }
    #[must_use]
    pub fn token(mut self, token: &str) -> ConfigBuilder<H, R, U, BF, S> {
        self.token = Some(token.to_string());
        self.cast()
    }

    #[must_use]
    pub fn servers(mut self, servers: &[String]) -> ConfigBuilder<H, R, U, T, BF> {
        self.servers = Some(servers.to_vec());
        self.cast()
    }
}

impl<H, R, U, T, S> ConfigBuilder<H, R, U, T, S>
where
    H: Requester,
    R: Send + Sync,
    U: Send + Sync,
    T: Send + Sync,
    S: Send + Sync,
{
    /// Fills `room_id` and `uid` by given `uid`, fetching `room_id` automatically.
    ///
    /// # Errors
    /// Returns an error when HTTP api request fails.
    pub async fn by_uid(mut self, uid: u64) -> Result<ConfigBuilder<H, BF, BF, T, S>, BuildError> {
        let resp: Resp<RoomQueryInner> = self
            .http
            .get_json(&*format!(
                "https://api.live.bilibili.com/bili/living_v2/{}",
                uid
            ))
            .await
            .map_err(BuildError)?;
        let room_id = resp.room_id();

        self.room_id = Some(room_id);
        self.uid = Some(uid);
        Ok(self.cast())
    }

    /// Fetches danmaku server configs & uris
    ///
    /// # Errors
    /// Returns an error when HTTP api request fails.
    pub async fn fetch_conf(mut self) -> Result<ConfigBuilder<H, R, U, BF, BF>, BuildError> {
        let resp: Resp<ConfQueryInner> = self
            .http
            .get_json("https://api.live.bilibili.com/room/v1/Danmu/getConf")
            .await
            .map_err(BuildError)?;

        self.token = Some(resp.token().to_string());
        self.servers = Some(resp.servers());
        Ok(self.cast())
    }
}

impl<H> ConfigBuilder<H, BF, BF, BF, BF> {
    /// Consumes the builder and returns [`StreamConfig`](StreamConfig)
    #[allow(clippy::missing_panics_doc)]
    pub fn build(self) -> StreamConfig {
        // SAFETY ensured by type state
        StreamConfig::new(
            self.room_id.unwrap(),
            self.uid.unwrap(),
            self.token.unwrap(),
            self.servers.unwrap(),
        )
    }
}
