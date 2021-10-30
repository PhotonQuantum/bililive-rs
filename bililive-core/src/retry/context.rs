use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;

use crate::config::StreamConfig;

/// Internal context for server picking during (re)connection.
///
/// Implements a round-robin policy for server selection.
#[derive(Debug, Clone)]
pub struct RetryContext {
    config: StreamConfig,
    cursor: Arc<AtomicUsize>,
}

impl RetryContext {
    /// Get the stream config.
    #[must_use]
    pub const fn config(&self) -> &StreamConfig {
        &self.config
    }
    /// Get the next server.
    #[allow(clippy::missing_panics_doc)]
    pub fn get(&mut self) -> &str {
        let cursor: usize = self
            .cursor
            .fetch_update(SeqCst, SeqCst, |i| {
                Some((i + 1) % self.config.servers().len())
            })
            .unwrap();
        &*self.config.servers()[cursor]
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
