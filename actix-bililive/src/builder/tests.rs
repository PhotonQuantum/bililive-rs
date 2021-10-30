use crate::core::config::Stream as StreamConfig;

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

#[actix_rt::test]
async fn must_build_real_config_tokio() {
    build_real_config(false).await;
}
