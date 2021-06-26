use std::time::Duration;

use anyhow::Result;
use futures::StreamExt;
use serde_json::Value;

use bililive_lib::{BililiveStream, ConfigBuilder};

async fn test_func(stream: &mut BililiveStream) {
    while let Some(e) = stream.next().await {
        match e {
            Ok(packet) => {
                println!("raw: {:?}", packet);
                if let Ok(json) = packet.json::<Value>() {
                    println!("json: {:?}", json);
                }
            }
            Err(e) => {
                println!("err: {:?}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = ConfigBuilder::new()
        .by_uid(419220)
        .await?
        .fetch_conf()
        .await?
        // .servers(&["wss://broadcastlv.chat.bilibili.com/sub".to_string()])
        .build()?;
    println!("room_id: {}", config.room_id);
    println!("uid: {}", config.uid);
    println!("token: {}", config.token);
    println!("servers: {:#?}", config.servers);

    let mut stream = BililiveStream::new(config);
    let _ = tokio::time::timeout(Duration::from_secs(10), test_func(&mut stream)).await;
    println!("disconnecting");
    stream.close();
    println!("joining");
    stream.join().await;
    // client.connect().await?;
    // println!("connected");
    // client.join().await?;
    // println!("joined");

    Ok(())
}
