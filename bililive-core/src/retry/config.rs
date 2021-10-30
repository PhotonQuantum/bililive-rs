use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::time::Duration;

use stream_reconnect::ReconnectOptions;

use super::policy::BEBIterator;

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
