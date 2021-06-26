use std::sync::{Arc, Mutex};

use tokio::time::Duration;

use crate::{Client, ConfigBuilder, Packet};

struct TestClient {
    client: Client,
    received: Arc<Mutex<Vec<Packet>>>,
}

impl TestClient {
    async fn new(uid: u64) -> Self {
        let received = Arc::new(Mutex::new(vec![]));
        let received_clone = received.clone();
        let client = ConfigBuilder::new()
            .by_uid(uid)
            .await
            .unwrap()
            .fetch_conf()
            .await
            .unwrap()
            .servers(&["wss://broadcastlv.chat.bilibili.com/sub".to_string()])
            .callback(Box::new(move |pack| {
                received_clone.lock().unwrap().push(pack);
            }))
            .build()
            .expect("unable to build client");
        Self { client, received }
    }
    fn received(&self) -> Vec<Packet> {
        self.received.lock().unwrap().clone()
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn must_run_client() {
    let mut test_client = TestClient::new(419220).await;
    test_client.client.connect().await.expect("can't connect");
    tokio::time::sleep(Duration::from_secs(5)).await;
    test_client.client.close().await.expect("can't close");
    assert!(test_client.received().len() > 0, "no package received");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn must_reuse_client() {
    let mut test_client = TestClient::new(419220).await;
    test_client.client.connect().await.expect("can't connect");
    tokio::time::sleep(Duration::from_secs(1)).await;
    test_client.client.close().await.expect("can't close");
    tokio::time::sleep(Duration::from_secs(1)).await;
    test_client.client.connect().await.expect("can't reconnect");
    tokio::time::sleep(Duration::from_secs(5)).await;
    assert!(test_client.received().len() > 0, "no package received");
    test_client.client.close().await.expect("can't close");
}
