use async_trait::async_trait;
use awc::Client;
use serde::de::DeserializeOwned;

use bililive_core::builder::Requester;

type BoxedError = Box<dyn std::error::Error>;

#[derive(Default)]
pub struct AWCClient(Client);

impl From<Client> for AWCClient {
    fn from(client: Client) -> Self {
        Self(client)
    }
}

#[async_trait(? Send)]
impl Requester for AWCClient {
    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, BoxedError> {
        Ok(self.0.get(url).send().await?.json().await?)
    }
}
