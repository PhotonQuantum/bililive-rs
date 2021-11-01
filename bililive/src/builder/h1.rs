use std::future::Future;
use std::pin::Pin;

use http_client::h1::H1Client as Client;
use http_client::HttpClient;
use serde::de::DeserializeOwned;

use crate::core::builder::Requester;

use super::BoxedError;

#[derive(Debug, Default)]
pub struct H1Client(Client);

impl From<Client> for H1Client {
    fn from(client: Client) -> Self {
        Self(client)
    }
}

impl Requester for H1Client {
    fn get_json<T: DeserializeOwned>(
        &self,
        url: &str,
    ) -> Pin<Box<dyn Future<Output = Result<T, BoxedError>> + Send + '_>> {
        let req = http_client::Request::get(url);
        Box::pin(async move {
            Ok(serde_json::from_slice(
                &*self.0.send(req).await?.body_bytes().await?,
            )?)
        })
    }
}
