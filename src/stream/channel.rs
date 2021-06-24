use tokio::sync::{broadcast, mpsc, oneshot};

use crate::{WsRxType, WsTxType};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) enum ConnEvent {
    Close,
    Failure,
}

pub(crate) struct ConnChan {
    tx_sender: mpsc::Sender<(WsTxType, oneshot::Sender<()>)>,
    tx_receiver: Option<mpsc::Receiver<(WsTxType, oneshot::Sender<()>)>>,
    rx_sender: mpsc::Sender<(WsRxType, oneshot::Sender<()>)>,
    rx_receiver: Option<mpsc::Receiver<(WsRxType, oneshot::Sender<()>)>>,
    event_sender: broadcast::Sender<ConnEvent>,
}
