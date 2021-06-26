use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;
use std::time::Duration;

use rand::distributions::Uniform;
use rand::{thread_rng, Rng};

pub type RetryPolicy = Arc<dyn Fn(u32) -> Option<Duration> + Send + Sync>;

#[derive(Debug, Clone)]
pub struct StreamConfig {
    // bilibili live room id (long)
    pub room_id: u64,
    // live user id
    pub uid: u64,
    // danmaku server token
    pub token: String,
    // danmaku server urls
    pub servers: Vec<String>,
    // retry config
    pub retry: RetryConfig,
}

impl StreamConfig {
    pub fn new(
        room_id: u64,
        uid: u64,
        token: &str,
        servers: &[String],
        retry: RetryConfig,
    ) -> Self {
        StreamConfig {
            room_id,
            uid,
            token: token.to_string(),
            servers: servers.to_vec(),
            retry,
        }
    }
}

#[derive(Clone)]
pub struct RetryConfig {
    // a connection lasts less than this period of time is considered a failure.
    pub min_conn_duration: Duration,
    // a policy function of retrying.
    // Input is a count of continuous failures.
    // The conn task will wait for such long and retry if a duration is returned.
    // Returning None causes the stream to close.
    pub retry_policy: RetryPolicy,
}

impl Debug for RetryConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("RetryConfig")
            .field("min_conn_duration", &self.min_conn_duration)
            .field("retry_policy", &String::from("<function>"))
            .finish()
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            min_conn_duration: Duration::from_secs(1),
            retry_policy: exponential_backoff_policy(Duration::from_secs(1), 5, 10),
        }
    }
}

// Return a exponential backoff retry policy.
// unit: unit duration of delay
// truncate: after a continuous failure of such counts, the delay stops increasing.
// fail: after a continuous failure of such counts, the connection closes.
// Truncate is expected to less than fail. Otherwise, a panic will occur.
pub fn exponential_backoff_policy(unit: Duration, truncate: u32, fail: u32) -> RetryPolicy {
    assert!(truncate < fail, "truncate >= fail");
    Arc::new(move |count| {
        if count >= fail {
            None
        } else {
            let max_delay = 2u32.pow(if count >= truncate { truncate } else { count });
            let between = Uniform::new_inclusive(0, max_delay);
            let units = thread_rng().sample(between);
            Some(unit * units)
        }
    })
}
