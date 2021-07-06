use anyhow::Result;
use futures::{Stream, StreamExt};
use log::info;
use serde_json::Value;

use bililive_lib::{ConfigBuilder, Packet, StreamError, connect_with_retry};

async fn test_func(stream: &mut (impl Stream<Item = Result<Packet, StreamError>> + Unpin)) {
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
        // .servers(&["wss://broadcastlv.chat.bilibili.com/sub".to_string()])
        .build()?;
    info!("room_id: {}", config.room_id);
    info!("uid: {}", config.uid);
    info!("token: {}", config.token);
    info!("servers: {:#?}", config.servers);

    // let mut stream = BililiveStreamNew::new(ws);
    let mut stream = connect_with_retry(config).await.unwrap();

    test_func(&mut stream).await;
    // let mut stream = BililiveStream::new(config);
    // let _ = tokio::time::timeout(Duration::from_secs(60), test_func(&mut stream)).await;
    // info!("disconnecting");
    // stream.close();
    // info!("joining");
    // stream.join().await;

    Ok(())
}
