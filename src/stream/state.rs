use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering::SeqCst;

#[derive(Debug)]
pub(crate) struct StreamStateStore {
    data: AtomicU8,
}

impl Default for StreamStateStore {
    fn default() -> Self {
        Self {
            data: AtomicU8::new(StreamState::default() as u8),
        }
    }
}

impl StreamStateStore {
    pub(crate) fn new() -> Self {
        Default::default()
    }
    pub(crate) fn load(&self) -> StreamState {
        self.data.load(SeqCst).into()
    }
    pub(crate) fn store(&self, state: StreamState) {
        self.data.store(state as u8, SeqCst);
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) enum StreamState {
    // The connection is active.
    Active = 1,
    // The connection is being established (connecting or retrying)
    Establishing = 2,
    // The connection does not exist anymore.
    Terminated = 0,
}

impl From<u8> for StreamState {
    fn from(v: u8) -> Self {
        match v {
            1 => StreamState::Active,
            2 => StreamState::Establishing,
            0 => StreamState::Terminated,
            _ => unreachable!(),
        }
    }
}

impl Default for StreamState {
    fn default() -> Self {
        Self::Establishing
    }
}

impl StreamState {
    pub(crate) fn is_active(&self) -> bool {
        matches!(self, StreamState::Active)
    }
}
