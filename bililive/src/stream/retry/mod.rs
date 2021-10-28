use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;

use bililive_core::config::Stream as StreamConfig;

#[macro_use]
mod imp;

#[cfg(feature = "tokio")]
mod tokio {
    //! `tokio` integration.
    impl_retry!(tokio);
}

#[cfg(feature = "async-std")]
mod async_std {
    //! `async_std` integration.
    impl_retry!(async_std);
}

/// Internal context for server picking during (re)connection.
///
/// Implements a round-robin policy for server selection.
#[derive(Debug, Clone)]
pub struct RetryContext {
    config: StreamConfig,
    cursor: Arc<AtomicUsize>,
}

impl RetryContext {
    /// Get the next server.
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
