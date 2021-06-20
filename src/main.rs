use anyhow::Result;
use serde_json::Value;

use bililive_lib::{ClientBuilder, Packet};

#[tokio::main]
async fn main() -> Result<()> {
    let callback = |pack: Packet| {
        println!("raw data: {:?}", pack);
        if let Ok(json) = pack.json::<Value>() {
            println!("json body: {:#?}", json);
        }
    };
    let mut client = ClientBuilder::new()
        .by_uid(419220)
        .await?
        .callback(Box::new(callback))
        .compression(false)
        .fetch_conf()
        .await?
        .servers(&["wss://broadcastlv.chat.bilibili.com/sub".to_string()])
        .build()?;
    println!("room_id: {}", client.room_id());
    println!("uid: {}", client.uid());
    println!("token: {}", client.token());
    println!("servers: {:#?}", client.servers());
    client.connect().await?;
    println!("connected");
    client.join().await?;
    println!("joined");

    Ok(())
}
