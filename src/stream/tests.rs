use std::time::Duration;

use futures::{Sink, SinkExt, Stream, StreamExt};

use crate::{
    connect, connect_with_retry, BililiveError, ConfigBuilder, Operation, Packet, Protocol,
    RetryConfig,
};

async fn test_stream(
    mut stream: impl Stream<Item = Result<Packet, BililiveError>>
        + Sink<Packet, Error = BililiveError>
        + Unpin,
) {
    let mut msg_count = 0;

    let stream_try = async {
        while let Some(msg) = stream.next().await {
            msg.expect("stream error");
            msg_count += 1;
        }
    };
    // err means timeout indicating there's no early stop on stream
    assert!(
        tokio::time::timeout(Duration::from_secs(3), stream_try)
            .await
            .is_err(),
        "stream error"
    );

    stream
        .send(Packet::new(Operation::HeartBeat, Protocol::Json, vec![]))
        .await
        .expect("sink error");
    let mut hb_resp_received = false;
    let stream_try = async {
        while let Some(msg) = stream.next().await {
            let msg = msg.expect("stream error");
            if msg.op() == Operation::HeartBeatResponse {
                hb_resp_received = true;
            }
        }
    };
    // err means timeout indicating there's no early stop on stream
    assert!(
        tokio::time::timeout(Duration::from_secs(1), stream_try)
            .await
            .is_err(),
        "stream error"
    );
    assert!(hb_resp_received, "no heart beat response received");

    stream.close().await.expect("unable to close stream");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn must_stream() {
    let config = ConfigBuilder::new()
        .by_uid(419220)
        .await
        .unwrap()
        .fetch_conf()
        .await
        .unwrap()
        .servers(&["wss://broadcastlv.chat.bilibili.com/sub".to_string()])
        .build()
        .expect("unable to build config");

    let stream = connect(config)
        .await
        .expect("unable to establish connection");
    test_stream(stream).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn must_retry_stream() {
    let config = ConfigBuilder::new()
        .by_uid(419220)
        .await
        .unwrap()
        .fetch_conf()
        .await
        .unwrap()
        .build()
        .expect("unable to build config");

    let stream = connect_with_retry(config, RetryConfig::default())
        .await
        .expect("unable to establish connection");
    test_stream(stream).await;
}
