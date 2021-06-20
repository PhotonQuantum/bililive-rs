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
        .room_id_by_uid(419220)
        .await?
        .callback(Box::new(callback))
        .compression(false)
        .build()?;
    client.connect().await?;
    client.join().await?;

    Ok(())
}
