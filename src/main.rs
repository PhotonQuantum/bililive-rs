use anyhow::Result;
use futures::{SinkExt, Stream, StreamExt};
use log::info;
use serde_json::{json, Value};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use bililive_lib::packet::raw::RawPacket;
use bililive_lib::{BililiveStreamNew, ConfigBuilder, Operation, Packet, Protocol, StreamError};

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

    let (mut ws, _) = connect_async("wss://broadcastlv.chat.bilibili.com/sub").await?;
    let room_enter = Message::binary(RawPacket::new(
        Operation::RoomEnter,
        Protocol::Json,
        serde_json::to_vec(&json!({
            "uid": config.uid,
            "roomid": config.room_id,
            "protover": 2,
            "platform": "web",
            "clientver": "1.8.2",
            "type": 2,
            "key": config.token
        }))?,
    ));
    ws.send(room_enter).await?;
    let mut stream = BililiveStreamNew::new(ws);

    test_func(&mut stream).await;
    // let mut stream = BililiveStream::new(config);
    // let _ = tokio::time::timeout(Duration::from_secs(60), test_func(&mut stream)).await;
    // info!("disconnecting");
    // stream.close();
    // info!("joining");
    // stream.join().await;

    Ok(())
}
