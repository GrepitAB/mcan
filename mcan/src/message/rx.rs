//! Messages received from the bus.

use super::*;

/// This trait is only implemented for the data sizes that the peripheral can be
/// configured to use. Only for the receive message format.
pub trait AnyMessage: super::AnyMessage {
    /// Create a transmission object from rx object
    fn as_tx_builder(&'_ self) -> tx::MessageBuilder<'_>;

    /// Timestamp counter value captured on start of frame reception
    fn timestamp(&self) -> u16;

    /// Index of the filter that accepted the frame. `None` if no filter
    /// matched, but the message was accepted due to peripheral-wide settings.
    fn filter_index(&self) -> Option<u8>;

    /// `true` if no filter matched, but the message was accepted due to
    /// peripheral-wide settings. See also [`Self::filter_index`]
    fn accepted_non_matching_frame(&self) -> bool;
}

impl<const N: usize> super::AnyMessage for Message<N>
where
    RawMessage<N>: super::AnyMessage,
{
    const REG: u8 = RawMessage::<N>::REG;
}

impl<const N: usize> super::Raw for Message<N> {
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

impl<const N: usize> AnyMessage for Message<N>
where
    Message<N>: super::AnyMessage,
{
    fn as_tx_builder(&'_ self) -> tx::MessageBuilder<'_> {
        tx::MessageBuilder {
            id: self.id(),
            frame_type: if self.fd_format() {
                tx::FrameType::FlexibleDatarate {
                    payload: self.data(),
                    bit_rate_switching: self.bit_rate_switching(),
                    force_error_state_indicator: false,
                }
            } else {
                tx::FrameType::Classic(if self.is_remote_frame() {
                    tx::ClassicFrameType::Remote {
                        desired_len: dlc_to_len(self.dlc(), self.fd_format()),
                    }
                } else {
                    tx::ClassicFrameType::Data(self.data())
                })
            },
            store_tx_event: None,
        }
    }

    fn timestamp(&self) -> u16 {
        self.0.header[1] as u16
    }

    fn filter_index(&self) -> Option<u8> {
        if self.accepted_non_matching_frame() {
            None
        } else {
            Some(((self.0.header[1] >> 24) & 0x7f) as u8)
        }
    }

    fn accepted_non_matching_frame(&self) -> bool {
        self.0.header[1] & (1 << 31) != 0 // ANMF
    }
}

/// RX message in the peripheral's representation
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Message<const N: usize>(pub(super) RawMessage<N>);
