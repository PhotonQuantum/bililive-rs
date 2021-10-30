#![allow(clippy::module_name_repetitions, clippy::future_not_send)]

pub use bililive_core as core;
pub use builder::ConfigBuilder;
pub use connect::*;

mod builder;
mod connect;
pub mod stream;
