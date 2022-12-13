//! Events for messages sent on the bus

use super::*;

impl Raw for TxEvent {
    fn id(&self) -> Id {
        self.0.id()
    }
    fn decoded_dlc(&self) -> usize {
        self.0.decoded_dlc()
    }
    fn dlc(&self) -> u8 {
        self.0.dlc()
    }
    fn fd_format(&self) -> bool {
        self.0.fd_format()
    }
    fn is_remote_frame(&self) -> bool {
        self.0.is_remote_frame()
    }
    fn data(&self) -> &[u8] {
        self.0.data()
    }
    fn is_extended(&self) -> bool {
        self.0.is_extended()
    }
    fn is_transmitter_error_passive(&self) -> bool {
        self.0.is_transmitter_error_passive()
    }
    fn bit_rate_switching(&self) -> bool {
        self.0.bit_rate_switching()
    }
}

/// TX event in the peripheral's representation
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct TxEvent(pub(super) RawMessage<0>);

impl TxEvent {
    /// Returns the message marker that was set in [`store_tx_event`]
    ///
    /// [`store_tx_event`]: crate::message::tx::MessageBuilder::store_tx_event
    pub fn message_marker(&self) -> u8 {
        (self.0.header[1] >> 24) as u8
    }

    /// Parse the event type field. Indicates whether cancellation was requested
    /// at the time transmission succeeded.
    pub fn event_type(&self) -> TxEventType {
        TxEventType::from((self.0.header[1] >> 22) & 3)
    }
}

/// Indicates whether cancellation was requested at the time transmission
/// succeeded
pub enum TxEventType {
    /// Unrecognized field value
    Reserved,
    /// Transmission was successful
    TxEvent = 1,
    /// Transmission was successful AND cancellation was requested
    ///
    /// This can happen if transmission had already started when the
    /// cancellation request was made.
    TxInSpiteOfCancellation = 2,
}

impl From<u32> for TxEventType {
    fn from(value: u32) -> Self {
        match value {
            1 => Self::TxEvent,
            2 => Self::TxInSpiteOfCancellation,
            _ => Self::Reserved,
        }
    }
}
