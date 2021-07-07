#[cfg(feature = "h1-client")]
use http_client::HttpClient;
use serde::de::DeserializeOwned;

use crate::errors::{ParseError, Result};

#[derive(Debug)]
pub enum HTTPClient {
    #[cfg(feature = "h1-client")]
    H1(http_client::h1::H1Client),
    #[cfg(feature = "reqwest")]
    Reqwest(reqwest::Client),
}

impl Default for HTTPClient {
    fn default() -> Self {
        #[cfg(feature = "reqwest")]
        return Self::Reqwest(reqwest::Client::new());
        #[cfg(feature = "h1-client")]
        return Self::H1(http_client::h1::H1Client::new());
    }
}

impl HTTPClient {
    #[cfg(feature = "h1-client")]
    fn get_h1(&self) -> &http_client::h1::H1Client {
        match self {
            #[cfg(feature = "h1-client")]
            HTTPClient::H1(c) => c,
            #[cfg(feature = "reqwest")]
            HTTPClient::Reqwest(_) => unreachable!(),
        }
    }

    #[cfg(feature = "reqwest")]
    fn get_reqwest(&self) -> &reqwest::Client {
        match self {
            #[cfg(feature = "h1-client")]
            HTTPClient::H1(_) => unreachable!(),
            #[cfg(feature = "reqwest")]
            HTTPClient::Reqwest(c) => c,
        }
    }

    pub(crate) async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        #[cfg(feature = "reqwest")]
        return Ok(serde_json::from_slice(
            &*self.get_reqwest().get(url).send().await?.bytes().await?,
        )
        .map_err(ParseError::JSON)?);
        #[cfg(feature = "h1-client")]
        {
            let req = http_client::Request::get(url);
            return Ok(serde_json::from_slice(
                &*self.get_h1().send(req).await?.body_bytes().await?,
            )
            .map_err(ParseError::JSON)?);
        }
    }
}
