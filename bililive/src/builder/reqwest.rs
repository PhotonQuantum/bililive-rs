use async_trait::async_trait;
use reqwest::Client;
use serde::de::DeserializeOwned;

use bililive_core::builder::Requester;

use super::BoxedError;

#[derive(Debug, Default)]
pub struct ReqwestClient(Client);

impl From<Client> for ReqwestClient {
    fn from(client: Client) -> Self {
        Self(client)
    }
}

#[async_trait]
impl Requester for ReqwestClient {
    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, BoxedError> {
        Ok(serde_json::from_slice(
            &*self.0.get(url).send().await?.bytes().await?,
        )?)
    }
}
