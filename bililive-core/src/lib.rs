//! A simple stream-based bilibili live danmaku implementation for Rust.
//!
//! This crate contains core traits, types and parsing implementations needed to build a
//! complete bilibili live client.
//!
//! If you need a batteries-included client, you may want to look at `bililive` or `actix-bililive`.
//!
//! ## Feature Flags
//! - `tokio-support` (default) - enable tokio support.
//! - `async-std-support` - enable async-std support.
//! - `not-send` - Remove `Send` constraints on traits and types. Useful for actix clients.

#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::module_name_repetitions,
    clippy::default_trait_access
)]

pub mod builder;
pub mod config;
pub mod errors;
pub mod packet;
pub mod retry;
pub mod stream;
