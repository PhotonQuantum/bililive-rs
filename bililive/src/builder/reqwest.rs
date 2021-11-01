use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;

use reqwest::Client;
use serde::de::DeserializeOwned;
use url::Url;

use crate::core::builder::Requester;

use super::BoxedError;

#[derive(Debug, Default)]
pub struct ReqwestClient(Client);

impl From<Client> for ReqwestClient {
    fn from(client: Client) -> Self {
        Self(client)
    }
}

impl Requester for ReqwestClient {
    fn get_json<T: DeserializeOwned>(
        &self,
        url: &str,
    ) -> Pin<Box<dyn Future<Output = Result<T, BoxedError>> + Send + '_>> {
        let url = Url::from_str(url).unwrap();
        Box::pin(async move {
            Ok(serde_json::from_slice(
                &*self.0.get(url).send().await?.bytes().await?,
            )?)
        })
    }
}
