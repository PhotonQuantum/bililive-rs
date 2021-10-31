use futures::StreamExt;
use log::info;
use serde_json::Value;

use bililive::core::retry::RetryConfig;
use bililive::ConfigBuilder;

async fn run() {
    pretty_env_logger::init();

    let config = ConfigBuilder::new()
        .by_uid(1602085)
        .await
        .unwrap()
        .fetch_conf()
        .await
        .unwrap()
        .build();
    info!("room_id: {}", config.room_id());
    info!("uid: {}", config.uid());
    info!("token: {}", config.token());
    info!("servers: {:#?}", config.servers());

    #[cfg(feature = "tokio")]
    let mut stream =
        bililive::connect::tokio::connect_with_retry(config.clone(), RetryConfig::default())
            .await
            .unwrap();
    #[cfg(feature = "async-std")]
    let mut stream =
        bililive::connect::async_std::connect_with_retry(config, RetryConfig::default())
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

fn main() {
    #[cfg(feature = "tokio")]
    {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        return runtime.block_on(run());
    }
    #[cfg(feature = "async-std")]
    return async_std::task::block_on(run());
}
