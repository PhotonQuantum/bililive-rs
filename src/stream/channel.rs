use tokio::sync::{broadcast, mpsc, oneshot};

use crate::{WsRxType, WsTxType};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) enum ConnEvent {
    Close,
    Failure,
}