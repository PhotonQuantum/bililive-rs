use futures::StreamExt;
use log::info;
use serde_json::Value;

use actix_bililive::{connect_with_retry, ConfigBuilder, RetryConfig};

#[actix_rt::main]
async fn main() {
    let config = ConfigBuilder::new()
        .by_uid(1602085)
        .await
        .unwrap()
        .fetch_conf()
        .await
        .unwrap()
        .build();

    let mut stream = connect_with_retry(config, RetryConfig::default())
        .await
        .unwrap();
    while let Some(e) = stream.next().await {
        match e {
            Ok(packet) => {
                info!("raw: {:?}", packet);
                if let Ok(json) = packet.json::<Value>() {
                    info!("json: {:?}", json);
                }
            }
            Err(e) => {
                info!("err: {:?}", e);
            }
        }
    }
}
