use std::time::Duration;

use rand::distributions::Uniform;
use rand::{thread_rng, Rng};

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
