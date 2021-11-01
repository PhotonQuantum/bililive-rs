use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;

use awc::http::Uri;
use awc::Client;
use serde::de::DeserializeOwned;

use crate::core::builder::Requester;

type BoxedError = Box<dyn std::error::Error>;

#[derive(Default)]
pub struct AWCClient(Client);

impl From<Client> for AWCClient {
    fn from(client: Client) -> Self {
        Self(client)
    }
}

impl Requester for AWCClient {
    fn get_json<T: DeserializeOwned>(
        &self,
        url: &str,
    ) -> Pin<Box<dyn Future<Output = Result<T, BoxedError>> + '_>> {
        let url = Uri::from_str(url).unwrap();
        Box::pin(async move { Ok(self.0.get(url).send().await?.json().await?) })
    }
}
