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
