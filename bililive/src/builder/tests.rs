use bililive_core::config::StreamConfig;

use super::ConfigBuilder;

pub(crate) async fn build_real_config(override_servers: bool) -> StreamConfig {
    let builder = ConfigBuilder::new()
        .by_uid(419220)
        .await
        .expect("unable to fetch room_id")
        .fetch_conf()
        .await
        .expect("unable to fetch server conf");
    let builder = if override_servers {
        builder.servers(&["wss://broadcastlv.chat.bilibili.com/sub".to_string()])
    } else {
        builder
    };
    builder.build()
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn must_build_real_config_tokio() {
    build_real_config(false).await;
}

#[cfg(feature = "async-std")]
#[async_std::test]
async fn must_build_real_config_async_std() {
    build_real_config(false).await;
}
