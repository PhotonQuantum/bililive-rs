use async_trait::async_trait;
use http_client::h1::H1Client as Client;
use http_client::HttpClient;
use serde::de::DeserializeOwned;

use bililive_core::builder::Requester;

use super::BoxedError;

#[derive(Debug, Default)]
pub struct H1Client(Client);

impl From<Client> for H1Client {
    fn from(client: Client) -> Self {
        Self(client)
    }
}

#[async_trait(?Send)]
impl Requester for H1Client {
    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, BoxedError> {
        let req = http_client::Request::get(url);
        Ok(serde_json::from_slice(
            &*self.0.send(req).await?.body_bytes().await?,
        )?)
    }
}
