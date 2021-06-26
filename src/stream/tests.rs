use std::time::Duration;

use futures::{SinkExt, StreamExt};

use crate::{BililiveStream, ConfigBuilder, Operation, Packet, Protocol};

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
async fn must_stream() {
    let config = ConfigBuilder::new()
        .by_uid(419220)
        .await
        .unwrap()
        .fetch_conf()
        .await
        .unwrap()
        .build()
        .expect("unable to build config");

    let mut stream = BililiveStream::new(config);
    let mut msg_count = 0;

    let stream_try = async {
        while let Some(msg) = stream.next().await {
            msg.expect("stream error");
            msg_count += 1;
        }
    };
    // err means timeout indicating there's no early stop on stream
    assert!(
        tokio::time::timeout(Duration::from_secs(10), stream_try)
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
        tokio::time::timeout(Duration::from_secs(3), stream_try)
            .await
            .is_err(),
        "stream error"
    );
    assert!(hb_resp_received, "no heart beat response received");

    stream.close();
    stream.join().await;
}
