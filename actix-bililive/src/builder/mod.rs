mod awc;
#[cfg(test)]
pub(crate) mod tests;

/// `bililive` stream config builder.
///
/// Stream config can be built via given live room parameters (room id and user id) & danmaku server configs (server token and list).
///
/// See the generic type [`ConfigBuilder`](bililive_core::builder::ConfigBuilder) for details.
///
/// # Helper methods
///
/// [`by_uid`](ConfigBuilder::by_uid) fetches room id by given user id.
///
/// [`fetch_conf`](ConfigBuilder::fetch_conf) fetches danmaku server token and list without any input parameter.
pub type ConfigBuilder<R, U, T, S> =
    bililive_core::builder::ConfigBuilder<awc::AWCClient, R, U, T, S>;
