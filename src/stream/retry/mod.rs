use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;

use crate::config::StreamConfig;

#[cfg(feature = "tokio")]
mod tokio;

#[cfg(feature = "async-std")]
mod async_std;

#[derive(Debug, Clone)]
pub struct RetryContext {
    config: StreamConfig,
    cursor: Arc<AtomicUsize>,
}

impl RetryContext {
    pub fn get(&mut self) -> &str {
        let cursor: usize = self
            .cursor
            .fetch_update(SeqCst, SeqCst, |i| {
                Some((i + 1) % self.config.servers.len())
            })
            .unwrap();
        &*self.config.servers[cursor]
    }
}

impl From<StreamConfig> for RetryContext {
    fn from(config: StreamConfig) -> Self {
        Self {
            config,
            cursor: Arc::new(Default::default()),
        }
    }
}
