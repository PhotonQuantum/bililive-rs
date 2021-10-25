use std::time::Duration;

use futures::{Sink, SinkExt, Stream, StreamExt};

use crate::builder::tests::build_real_config;
use crate::{BililiveError, Operation, Packet, Protocol, RetryConfig};

macro_rules! must_future_timeout {
    ($secs: literal, $future: expr) => {{
        let fut = $future;
        if cfg!(feature = "tokio") {
            #[cfg(feature = "tokio")]
            assert!(
                tokio::time::timeout(Duration::from_secs($secs), fut)
                    .await
                    .is_err(),
                "future not timeout"
            );
        } else {
            #[cfg(feature = "async-std")]
            assert!(
                async_std::future::timeout(Duration::from_secs($secs), fut)
                    .await
                    .is_err(),
                "future not timeout"
            );
        };
    }};
}

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
    must_future_timeout!(3, stream_try);

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
    must_future_timeout!(1, stream_try);
    assert!(hb_resp_received, "no heart beat response received");

    stream.close().await.expect("unable to close stream");
}

async fn test_stream_heartbeat(
    mut stream: impl Stream<Item = Result<Packet, BililiveError>>
        + Sink<Packet, Error = BililiveError>
        + Unpin,
) {
    let stream_try = async {
        while let Some(Ok(_)) = stream.next().await {}
        panic!("connection closed (heartbeat not sent)");
    };
    // err means timeout indicating there's no early stop on stream
    must_future_timeout!(120, stream_try);

    stream.close().await.expect("unable to close stream");
}

#[cfg(feature = "tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn must_stream_tokio() {
    let config = build_real_config(true).await;

    let stream = crate::connect::tokio::connect(config)
        .await
        .expect("unable to establish connection");
    test_stream(stream).await;
}

#[cfg(feature = "tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn must_retry_stream_tokio() {
    let config = build_real_config(false).await;

    let stream = crate::connect::tokio::connect_with_retry(config, RetryConfig::default())
        .await
        .expect("unable to establish connection");
    test_stream(stream).await;
}

#[cfg(feature = "async-std")]
#[async_std::test]
async fn must_stream_async_std() {
    let config = build_real_config(true).await;

    let stream = crate::connect::async_std::connect(config)
        .await
        .expect("unable to establish connection");
    test_stream(stream).await;
}

#[cfg(feature = "async-std")]
#[async_std::test]
async fn must_retry_async_std() {
    let config = build_real_config(false).await;

    let stream = crate::connect::async_std::connect_with_retry(config, RetryConfig::default())
        .await
        .expect("unable to establish connection");
    test_stream(stream).await;
}

#[cfg(feature = "tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn must_hb_tokio() {
    if option_env!("FAST_TEST").is_some() {
        return;
    }

    let config = build_real_config(true).await;

    let stream = crate::connect::tokio::connect(config)
        .await
        .expect("unable to establish connection");
    test_stream_heartbeat(stream).await;
}

#[cfg(feature = "async-std")]
#[async_std::test]
async fn must_hb_async_std() {
    if option_env!("FAST_TEST").is_some() {
        return;
    }

    let config = build_real_config(true).await;

    let stream = crate::connect::async_std::connect(config)
        .await
        .expect("unable to establish connection");
    test_stream_heartbeat(stream).await;
}
