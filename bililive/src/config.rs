//! Configuration types.
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::time::Duration;

use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use stream_reconnect::ReconnectOptions;

/// The configuration for bilibili live stream connection.
#[derive(Debug, Clone)]
pub struct StreamConfig(Box<Inner>);

impl StreamConfig {
    pub fn new(room_id: u64, uid: u64, token: String, servers: Vec<String>) -> Self {
        Self(Box::new(Inner {
            room_id,
            uid,
            token,
            servers,
        }))
    }
}

impl StreamConfig {
    pub const fn room_id(&self) -> u64 {
        self.0.room_id
    }
    pub const fn uid(&self) -> u64 {
        self.0.uid
    }
    pub fn token(&self) -> &str {
        &self.0.token
    }
    pub fn servers(&self) -> &[String] {
        &self.0.servers
    }
}

#[derive(Debug, Clone)]
struct Inner {
    /// Live room id (long version).
    room_id: u64,
    /// Live room user id.
    uid: u64,
    /// Danmaku server token.
    token: String,
    /// Danmaku server urls.
    servers: Vec<String>,
}

/// The configuration for retry behavior.
#[derive(Clone)]
pub struct RetryConfig(ReconnectOptions);

impl RetryConfig {
    /// Create a retry configuration with given `duration_generator`.
    ///
    /// `duration_generator` is a function that returns a duration iterator.
    /// Each item yielded by the iterator indicates the delay time before next connection attempt after a disconnection occurs.
    /// If `None` is returned, the stream fails.
    ///
    /// The `default` implementation uses [`BEBIterator`](BEBIterator).
    pub fn new<F, I, IN>(duration_generator: F) -> Self
    where
        F: 'static + Send + Sync + Fn() -> IN,
        I: 'static + Send + Sync + Iterator<Item = Duration>,
        IN: IntoIterator<IntoIter = I, Item = Duration>,
    {
        Self(ReconnectOptions::new().with_retries_generator(duration_generator))
    }
}

impl From<RetryConfig> for ReconnectOptions {
    fn from(o: RetryConfig) -> Self {
        o.0
    }
}

impl Debug for RetryConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("RetryConfig")
            .field(&"<ReconnectOptions>")
            .finish()
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::new(BEBIterator::default)
    }
}

/// An exponential backoff retry policy.
#[derive(Debug, Clone)]
pub struct BEBIterator {
    unit: Duration,
    truncate: u32,
    fail: u32,
    count: u32,
}

impl Default for BEBIterator {
    fn default() -> Self {
        Self::new(Duration::from_secs(1), 5, 10)
    }
}

impl BEBIterator {
    /// Create an exponential backoff retry policy
    ///
    /// # Arguments
    ///
    /// * `unit`: unit duration of delay.
    /// * `truncate`: after a continuous failure of such counts, the delay stops increasing.
    /// * `fail`: after a continuous failure of such counts, the connection closes.
    ///
    /// returns: `BEBIterator`
    ///
    /// # Panics
    ///
    /// Truncate is expected to less than fail. Otherwise, a panic will occur.
    #[must_use]
    pub fn new(unit: Duration, truncate: u32, fail: u32) -> Self {
        assert!(truncate < fail, "truncate >= fail");
        Self {
            unit,
            truncate,
            fail,
            count: 0,
        }
    }
}

impl Iterator for BEBIterator {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= self.fail {
            None
        } else {
            let max_delay = 2_u32.pow(if self.count >= self.truncate {
                self.truncate
            } else {
                self.count
            });
            let between = Uniform::new_inclusive(0, max_delay * 100);
            let units = thread_rng().sample(between);
            Some(self.unit * units / 100)
        }
    }
}
