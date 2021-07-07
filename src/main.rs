use anyhow::Result;
use futures::{Stream, StreamExt};
use log::info;
use serde_json::Value;

use bililive_lib::tokio::connect_with_retry;
use bililive_lib::{BililiveError, ConfigBuilder, Packet, RetryConfig};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let config = ConfigBuilder::new()
        .by_uid(1602085)
        .await?
        .fetch_conf()
        .await?
        .build()?;
    info!("room_id: {}", config.room_id);
    info!("uid: {}", config.uid);
    info!("token: {}", config.token);
    info!("servers: {:#?}", config.servers);

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

    Ok(())
}
