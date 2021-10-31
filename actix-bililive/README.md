# actix-bililive

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/PhotonQuantum/bililive-rs/Test?style=flat-square)](https://github.com/PhotonQuantum/bililive-rs/actions/workflows/test.yml)
[![crates.io](https://img.shields.io/crates/v/actix-bililive?style=flat-square)](https://crates.io/crates/actix-bililive)
[![Documentation](https://img.shields.io/docsrs/actix-bililive?style=flat-square)](https://docs.rs/actix-bililive)

A simple stream-based bilibili live client library for the Actix ecosystem.

*Minimum supported rust version: 1.56.0*

## Runtime Support

This crate supports `actix-rt` (single-threaded `tokio`) runtime.

## Features

- Ergonomic `Stream`/`Sink` interface.
- Easy establishment of connection via given live room id.
- Handles heartbeat packets automatically.
- Auto retry when connection fails (optional).
- Decompresses `Zlib` payloads automatically.

## Example

```rust
use actix_bililive::{ConfigBuilder, RetryConfig, connect_with_retry};

use futures::StreamExt;
use log::info;
use serde_json::Value;

let config = ConfigBuilder::new()
    .by_uid(1602085)
    .await
    .unwrap()
    .fetch_conf()
    .await
    .unwrap()
    .build();

let mut stream = connect_with_retry(config, RetryConfig::default()).await.unwrap();
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
```