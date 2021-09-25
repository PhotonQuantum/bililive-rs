//! A simple stream-based bilibili live client library.
//!
//! *Minimum supported rust version: 1.53.0*
//!
//! ## Runtime Support
//!
//! This crate supports both `tokio` and `async-std` runtime.
//!
//! `tokio` support is enabled by default. While used on an `async-std` runtime, change the corresponding dependency in Cargo.toml to
//!
//! ```toml
//! bililive = { version = "0.1", default-features = false, features = ["async-native-tls"] }
//! ```
//!
//! See `Crates Features` section for more.
//!
//! ## Features
//!
//! - Ergonomic `Stream`/`Sink` interface.
//! - Easy establishment of connection via given live room id.
//! - Handles heartbeat packets automatically.
//! - Auto retry when connection fails (optional).
//! - Decompresses `Zlib` payloads automatically.
//!
//! ## Example
//!
//! ```rust
//! # #[cfg(feature = "tokio")]
//! use bililive::connect::tokio::connect_with_retry;
//! use bililive::errors::Result;
//! use bililive::{ConfigBuilder, RetryConfig};
//!
//! use futures::StreamExt;
//! use log::info;
//! use serde_json::Value;
//!
//! # #[cfg(feature = "tokio")]
//! # async fn test() -> Result<()> {
//! let config = ConfigBuilder::new()
//!     .by_uid(1602085)
//!     .await?
//!     .fetch_conf()
//!     .await?
//!     .build()?;
//!
//! let mut stream = connect_with_retry(config, RetryConfig::default()).await?;
//! while let Some(e) = stream.next().await {
//!     match e {
//!         Ok(packet) => {
//!             info!("raw: {:?}", packet);
//!             if let Ok(json) = packet.json::<Value>() {
//!                 info!("json: {:?}", json);
//!             }
//!         }
//!         Err(e) => {
//!             info!("err: {:?}", e);
//!         }
//!     }
//! }
//! #
//! # Ok(())
//! # }
//! ```
//!
//! ## Crate Features
//!
//! * `tokio-native-tls`(default): Enables `tokio` support with TLS implemented
//! via [tokio-native-tls](https://crates.io/crates/tokio-native-tls).
//! * `tokio-rustls-native-certs`: Enables `tokio` support with TLS implemented
//! via [tokio-rustls](https://crates.io/crates/tokio-rustls) and uses native system certificates found
//! with [rustls-native-certs](https://github.com/rustls/rustls-native-certs).
//! * `tokio-rustls-webpki-roots`: Enables `tokio` support with TLS implemented
//! via [tokio-rustls](https://crates.io/crates/tokio-rustls) and uses the
//! certificates [webpki-roots](https://github.com/rustls/webpki-roots) provides.
//! * `async-native-tls`: Enables `async_std` support with TLS implemented
//! via [async-native-tls](https://crates.io/crates/async-native-tls).

#[doc(inline)]
pub use crate::builder::ConfigBuilder;
#[doc(inline)]
pub use crate::config::{RetryConfig, StreamConfig};
#[doc(inline)]
pub use crate::errors::BililiveError;
#[doc(inline)]
pub use crate::packet::{Operation, Packet, Protocol};
#[doc(inline)]
pub use crate::stream::BililiveStream;

#[macro_use]
mod utils;
pub mod builder;
pub mod config;
pub mod connect;
pub mod errors;
pub mod packet;
pub mod stream;
