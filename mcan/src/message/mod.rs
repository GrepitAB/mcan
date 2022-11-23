//! Handling of messages/frames

pub mod rx;
pub mod tx;
mod tx_event;

pub use tx_event::{TxEvent, TxEventType};

use core::cmp::min;
use embedded_can::{ExtendedId, Frame, Id, StandardId};

/// This trait is only implemented for the data sizes that the peripheral can be
/// configured to use.
pub trait AnyMessage: Copy + Raw {
    /// The value of the data size field that indicates this data size
    const REG: u8;
}

macro_rules! impl_any_message {
    ($len:literal, $reg:literal) => {
        impl AnyMessage for RawMessage<$len> {
            const REG: u8 = $reg;
        }
    };
}

impl_any_message!(8, 0);
impl_any_message!(12, 1);
impl_any_message!(16, 2);
impl_any_message!(20, 3);
impl_any_message!(24, 4);
impl_any_message!(32, 5);
impl_any_message!(48, 6);
impl_any_message!(64, 7);

/// Data does not fit in the backing buffer
#[derive(Debug)]
pub struct TooMuchData;

/// CAN frame/message.
pub enum Message<const N: usize> {
    /// Message received from a CAN bus
    Rx(rx::Message<N>),
    /// Message that may be transmitted to a CAN bus
    Tx(tx::Message<N>),
}

impl<const N: usize> Message<N> {
    fn raw(&self) -> &RawMessage<N> {
        match self {
            Self::Rx(rx::Message(m)) | Self::Tx(tx::Message(m)) => m,
        }
    }
}

impl<const N: usize> Frame for Message<N> {
    fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
        tx::MessageBuilder {
            id: id.into(),
            frame_type: tx::FrameType::Classic(tx::ClassicFrameType::Data(data)),
            store_tx_event: None,
        }
        .build()
        .ok()
        .map(Self::Tx)
    }

    fn new_remote(id: impl Into<Id>, dlc: usize) -> Option<Self> {
        if dlc > 15 {
            return None;
        }
        tx::MessageBuilder {
            id: id.into(),
            frame_type: tx::FrameType::Classic(tx::ClassicFrameType::Remote {
                desired_len: dlc_to_len(dlc as u8, false),
            }),
            store_tx_event: None,
        }
        .build()
        .ok()
        .map(Self::Tx)
    }

    fn is_extended(&self) -> bool {
        self.raw().is_extended()
    }

    fn is_remote_frame(&self) -> bool {
        self.raw().is_remote_frame()
    }

    fn id(&self) -> Id {
        self.raw().id()
    }

    fn dlc(&self) -> usize {
        self.raw().dlc().into()
    }

    fn data(&self) -> &[u8] {
        self.raw().data()
    }
}

/// RX or TX message in the peripheral's representation
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct RawMessage<const N: usize> {
    header: [u32; 2],
    data: [u8; N],
}

/// Common functionality for all raw messages. This is a trait instead of
/// directly associated methods to allow the message size to be erased.
pub trait Raw {
    /// Returns the CAN identifier of the message
    fn id(&self) -> Id;
    /// Data length in bytes
    fn decoded_dlc(&self) -> usize;
    /// Data length code
    fn dlc(&self) -> u8;
    /// True if the header indicates that the frame uses the CAN FD format
    fn fd_format(&self) -> bool;
    /// Remote Transmission Request
    fn is_remote_frame(&self) -> bool;
    /// Data field
    fn data(&self) -> &[u8];
    /// Check if the frame uses and extended (29-bit) ID
    fn is_extended(&self) -> bool;
    /// `true` if the sender of the message indicates that it is in "error
    /// passive" state.
    fn is_transmitter_error_passive(&self) -> bool;
    /// `true` if bit rate switching is used
    fn bit_rate_switching(&self) -> bool;
}

impl<const N: usize> Raw for RawMessage<N> {
    fn id(&self) -> Id {
        if self.is_extended() {
            // The mask ensures the ID is in range for a 29-bit integer
            Id::Extended(unsafe {
                ExtendedId::new_unchecked(self.header[0] & ExtendedId::MAX.as_raw())
            })
        } else {
            // The mask ensures the ID is in range for a 11-bit integer
            Id::Standard(unsafe {
                StandardId::new_unchecked((self.header[0] >> 18) as u16 & StandardId::MAX.as_raw())
            })
        }
    }

    fn decoded_dlc(&self) -> usize {
        dlc_to_len(self.dlc(), self.fd_format())
    }

    fn dlc(&self) -> u8 {
        ((self.header[1] >> 16) & 0xf) as u8 // DLC
    }

    fn fd_format(&self) -> bool {
        self.header[1] & (1 << 21) != 0 // FDF
    }

    fn is_remote_frame(&self) -> bool {
        self.header[0] & (1 << 29) != 0 // RTR
    }

    fn data(&self) -> &[u8] {
        if !self.is_remote_frame() {
            self.data
                .get(..min(self.decoded_dlc(), self.data.len()))
                .unwrap_or(&[])
        } else {
            &[]
        }
    }

    fn is_extended(&self) -> bool {
        self.header[0] & (1 << 30) != 0 // XTD
    }

    fn is_transmitter_error_passive(&self) -> bool {
        self.header[0] & (1 << 31) != 0 // ESI
    }

    fn bit_rate_switching(&self) -> bool {
        self.header[1] & (1 << 20) != 0 // BRS
    }
}

/// Finds the smallest data length code that encodes at least len bytes
fn len_to_dlc(len: usize, fd_format: bool) -> Result<u8, TooMuchData> {
    if fd_format {
        match len as u8 {
            0..=8 => Ok(len as u8),
            9..=12 => Ok(9),
            13..=16 => Ok(10),
            17..=20 => Ok(11),
            21..=24 => Ok(12),
            25..=32 => Ok(13),
            33..=48 => Ok(14),
            49..=64 => Ok(15),
            65.. => Err(TooMuchData),
        }
    } else {
        match len as u8 {
            0..=8 => Ok(len as u8),
            9.. => Err(TooMuchData),
        }
    }
}

/// Converts data length code to a length in bytes
fn dlc_to_len(dlc: u8, fd_format: bool) -> usize {
    if fd_format {
        match dlc {
            0..=8 => dlc.into(),
            9 => 12,
            10 => 16,
            11 => 20,
            12 => 24,
            13 => 32,
            14 => 48,
            15.. => 64,
        }
    } else {
        match dlc {
            0..=8 => dlc.into(),
            9.. => 8,
        }
    }
}
