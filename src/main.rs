use anyhow::Result;
use serde_json::Value;

use bililive_lib::{ConfigBuilder, Packet, BililiveStream, StreamError};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let callback = |pack: Packet| {
        println!("raw data: {:?}", pack);
        if let Ok(json) = pack.json::<Value>() {
            println!("json body: {:#?}", json);
        }
    };
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
    // client.connect().await?;
    // println!("connected");
    // client.join().await?;
    // println!("joined");
    while let Some(e) = stream.next().await {
        match e{
            Ok(packet) => {
                println!("raw: {:?}", packet);
                if let Ok(json) = packet.json::<Value>() {
                    println!("json: {:?}", json);
                }
            }
            Err(e) => {
                println!("err: {:?}", e)
            }
        }
    }

    Ok(())
}
