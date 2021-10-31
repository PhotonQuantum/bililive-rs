//! A simple stream-based bilibili live client library backed by [awc](https://github.com/actix/actix-web/tree/master/awc).
//!
//! *Minimum supported rust version: 1.56.0*
//!
//! ## Runtime Support
//!
//! This crate supports `actix-rt` (single-threaded `tokio`) runtime.
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
//! use actix_bililive::{ConfigBuilder, RetryConfig, connect_with_retry};
//!
//! use futures::StreamExt;
//! use log::info;
//! use serde_json::Value;
//!
//! # async fn test() {
//! let config = ConfigBuilder::new()
//!     .by_uid(1602085)
//!     .await
//!     .unwrap()
//!     .fetch_conf()
//!     .await
//!     .unwrap()
//!     .build();
//!
//! let mut stream = connect_with_retry(config, RetryConfig::default()).await.unwrap();
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
//! # }
//! ```

#![allow(clippy::module_name_repetitions, clippy::future_not_send)]

pub use bililive_core as core;
#[doc(inline)]
pub use builder::ConfigBuilder;
pub use connect::{connect, connect_with_retry};

pub use crate::core::errors;
pub use crate::core::packet::*;
pub use crate::core::retry::RetryConfig;

mod builder;
mod connect;
pub mod stream;
