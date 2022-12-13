//! Messages to be sent on the bus

use super::*;

/// This trait is only implemented for the data sizes that the peripheral can be
/// configured to use. Only for the transmit message format.
pub trait AnyMessage: super::AnyMessage {
    /// Constructs the message described by `m`
    fn new(m: MessageBuilder) -> Result<Self, TooMuchData>;
}

impl<const N: usize> super::AnyMessage for Message<N>
where
    RawMessage<N>: super::AnyMessage,
{
    const REG: u8 = RawMessage::<N>::REG;
}

impl<const N: usize> Raw for Message<N> {
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
    fn new(m: MessageBuilder) -> Result<Self, TooMuchData> {
        m.build()
    }
}

/// TX message in the peripheral's representation
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Message<const N: usize>(pub(super) RawMessage<N>);

/// Selects the type of the Classic CAN frame.
pub enum ClassicFrameType<'a> {
    /// 0-8 byte message payload
    Data(&'a [u8]),
    /// Requests transmission of the identified frame
    Remote {
        /// Length, in bytes, of the requested frame
        desired_len: usize,
    },
}

/// Selects frame type along with the valid payload type and configuration
/// specific to the chosen format.
pub enum FrameType<'a> {
    /// Classic CAN
    Classic(ClassicFrameType<'a>),
    /// CAN FD frame. Note that the peripheral must be initialized with CAN FD
    /// enabled to support this format.
    FlexibleDatarate {
        /// 0-64 byte message payload. The payload must not be bigger than the
        /// maximum payload size chosen in [`Capacities::TxMessage`].
        ///
        /// [`Capacities::TxMessage`]: crate::messageram::Capacities
        payload: &'a [u8],
        /// Parts of the frame are transmitted at a higher bit rate. Note that
        /// bit rate switching must be enabled in the peripheral configuration
        /// as well.
        bit_rate_switching: bool,
        /// If `true`, the error state indicator of the message will indicate
        /// 'error passive'. If `false`, the actual state of the
        /// peripheral will be indicated.
        force_error_state_indicator: bool,
    },
}

/// Describes a CAN message/frame that is not yet converted to the
/// representation the peripheral understands.
pub struct MessageBuilder<'a> {
    /// CAN identifier for the frame
    pub id: Id,
    /// Message frame type with a payload
    pub frame_type: FrameType<'a>,
    /// If `Some(marker)`, this message will store an event identified by
    /// `marker` in the TX event queue.
    pub store_tx_event: Option<u8>,
}

impl<'a> MessageBuilder<'a> {
    /// Create the message in the format required by the peripheral.
    pub fn build<const N: usize>(self) -> Result<Message<N>, TooMuchData> {
        let mut data = [0; N];

        let mut copy_payload = |d: &[u8]| {
            if d.len() > N {
                return Err(TooMuchData);
            }
            data[..d.len()].copy_from_slice(d);
            Ok(())
        };

        let id_field = match self.id {
            Id::Standard(id) => (id.as_raw() as u32) << 18,
            Id::Extended(id) => id.as_raw(),
        };
        let xtd = matches!(self.id, Id::Extended(_));
        let (fdf, brs, esi, rtr, len) = match self.frame_type {
            FrameType::Classic(payload) => {
                let (rtr, len) = match payload {
                    ClassicFrameType::Data(payload) => {
                        copy_payload(payload)?;
                        (false, payload.len())
                    }
                    ClassicFrameType::Remote { desired_len } => (true, desired_len),
                };
                (false, false, false, rtr, len)
            }
            FrameType::FlexibleDatarate {
                payload,
                bit_rate_switching: brs,
                force_error_state_indicator: esi,
            } => {
                copy_payload(payload)?;
                (true, brs, esi, false, payload.len())
            }
        };
        let dlc = len_to_dlc(len, fdf)?;
        let efc = self.store_tx_event.is_some();
        let mm = self.store_tx_event.unwrap_or(0);

        let t0 = id_field | (rtr as u32) << 29 | (xtd as u32) << 30 | (esi as u32) << 31;
        let t1 = (((dlc & 0xf) as u32) << 16)
            | ((brs as u32) << 20)
            | ((fdf as u32) << 21)
            | ((efc as u32) << 23)
            | ((mm as u32) << 24);
        Ok(Message(RawMessage {
            header: [t0, t1],
            data,
        }))
    }
}
