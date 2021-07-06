use std::sync::Arc;
use std::task::{Wake, Waker};

use futures::task::AtomicWaker;

// When reading the stream, a poll_ready is executed to ensure that all pending write op including
// heartbeat is completed.
// Therefore, we need to wake the task on which stream is polled in poll_ready (and poll_flush).
// WakerProxy is a waker dispatcher. It will dispatch a wake op to both wakers (rx & tx), such that
// both stream task and sink task can be waken and no starvation will occur.
#[derive(Debug, Default)]
pub struct WakerProxy {
    tx_waker: AtomicWaker,
    rx_waker: AtomicWaker,
}

impl WakerProxy {
    pub fn rx(&self, waker: &Waker) {
        self.rx_waker.register(waker);
    }
    pub fn tx(&self, waker: &Waker) {
        self.tx_waker.register(waker);
    }
}

impl Wake for WakerProxy {
    fn wake(self: Arc<Self>) {
        self.rx_waker.wake();
        self.tx_waker.wake();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.rx_waker.wake();
        self.tx_waker.wake();
    }
}
