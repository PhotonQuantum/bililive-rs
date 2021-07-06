use anyhow::Result;
use futures::{Stream, StreamExt};
use log::info;
use serde_json::Value;

use bililive_lib::{connect_with_retry, BililiveError, ConfigBuilder, Packet, RetryConfig};

async fn test_func(
    stream: &mut (impl Stream<Item = Result<Packet, BililiveError>> + Send + Unpin),
) {
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

    test_func(&mut stream).await;

    Ok(())
}
