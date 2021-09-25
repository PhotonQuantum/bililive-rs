# bililive-rs

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/PhotonQuantum/bililive-rs/Test?style=flat-square)](https://github.com/PhotonQuantum/bililive-rs/actions/workflows/test.yml)
[![crates.io](https://img.shields.io/crates/v/bililive?style=flat-square)](https://crates.io/crates/bililive)
[![Documentation](https://img.shields.io/docsrs/bililive?style=flat-square)](https://docs.rs/bililive)

A simple stream-based bilibili live client library.

To use with your project, add the following to your Cargo.toml:

```
bililive = "0.1"
```

*Minimum supported rust version: 1.53.0*

## Runtime Support

This crate supports both `tokio` and `async-std` runtime.

`tokio` support is enabled by default. While used on an `async-std` runtime, change the corresponding dependency in Cargo.toml to

```
bililive = { version = "0.1", default-features = false, features = ["async-native-tls"] }
```

See `Crates Features` section for more.

## Features

- Ergonomic `Stream`/`Sink` interface.
- Easy establishment of connection via given live room id.
- Handles heartbeat packets automatically.
- Auto retry when connection fails (optional).
- Decompresses `ZLib` payloads automatically.

## Example

```rust
use bililive::connect::tokio::connect_with_retry;
use bililive::errors::Result;
use bililive::{ConfigBuilder, RetryConfig};

use futures::StreamExt;
use serde_json::Value;

let config = ConfigBuilder::new()
    .by_uid(1602085)
    .await?
    .fetch_conf()
    .await?
    .build()?;

let mut stream = connect_with_retry(config, RetryConfig::default()).await?;
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

## Crate Features

- `tokio-native-tls`(default): Enables `tokio` support with TLS implemented
  via [tokio-native-tls](https://crates.io/crates/tokio-native-tls).
- `tokio-rustls-native-certs`: Enables `tokio` support with TLS implemented
  via [tokio-rustls](https://crates.io/crates/tokio-rustls) and uses native system certificates found
  with [rustls-native-certs](https://github.com/rustls/rustls-native-certs).
- `tokio-rustls-webpki-roots`: Enables `tokio` support with TLS implemented
  via [tokio-rustls](https://crates.io/crates/tokio-rustls) and uses the
  certificates [webpki-roots](https://github.com/rustls/webpki-roots) provides.
- `async-native-tls`: Enables `async_std` support with TLS implemented
  via [async-native-tls](https://crates.io/crates/async-native-tls).