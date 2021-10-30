//! `bililive` config builder.
//!
//! Stream config can be built via given live room parameters (room id and user id) & danmaku server configs (server token and list).
//!
//! # Helper methods
//!
//! [`by_uid`](ConfigBuilder::by_uid) fetches room id by given user id.
//!
//! [`fetch_conf`](ConfigBuilder::fetch_conf) fetches danmaku server token and list without any input parameter.
//!
//! # Example
//!
//! ```rust
//! # use std::future::Future;
//! #
//! # use bililive::ConfigBuilder;
//! # use bililive_core::errors::Build;
//! #
//! # let fut = async {
//! # Ok::<_, Build>(
//! ConfigBuilder::new()
//!     .by_uid(1472906636)
//!     .await?
//!     .fetch_conf()
//!     .await?
//!     .build()
//! # )
//! # };
//! #
//! # tokio_test::block_on(fut).unwrap();
//! ```

#[cfg(feature = "h1-client")]
mod h1;
#[cfg(feature = "reqwest")]
mod reqwest;
#[cfg(test)]
pub(crate) mod tests;

type BoxedError = Box<dyn std::error::Error>;

#[cfg(feature = "reqwest")]
pub type ConfigBuilder<R, U, T, S> =
    bililive_core::builder::Config<reqwest::ReqwestClient, R, U, T, S>;

#[cfg(feature = "h1-client")]
#[cfg(not(feature = "reqwest"))]
pub type ConfigBuilder<R, U, T, S> = bililive_core::builder::Config<h1::H1Client, R, U, T, S>;
