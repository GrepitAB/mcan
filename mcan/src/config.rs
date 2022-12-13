//! CAN bus configuration

pub use crate::reg::{self, tscc::TSS_A as TimeStampSelect};
use core::ops::RangeInclusive;
use fugit::HertzU32;

/// Configuration for the CAN bus
#[derive(Copy, Clone)]
pub struct CanConfig {
    /// Run peripheral in CAN-FD mode
    pub mode: Mode,
    /// Modes of testing
    pub loopback: bool,
    /// Bit timing parameters for everything except the data phase of bit rate
    /// switched FD frames.
    pub nominal_timing: BitTiming,
    /// Timestamp configuration
    pub timestamp: Timestamp,
    /// RX Fifo 0
    pub rx_fifo_0: RxFifoConfig,
    /// RX Fifo 1
    pub rx_fifo_1: RxFifoConfig,
    /// Tx configuration
    pub tx: TxConfig,
}

/// Denotes a TX related configuration
#[derive(Default, Copy, Clone)]
pub struct TxConfig {
    /// Denotes TX Event queue fullness required to trigger a corresponding
    /// interrupt
    ///
    /// Any value greater than 32 is interpreted as 32; 0 means that interrupt
    /// is disabled
    pub tx_event_fifo_watermark: u8,
    /// TX queue submode
    pub tx_queue_submode: TxQueueMode,
}

/// Bit-timing parameters
///
/// The bit time is determined by
/// - the time quantum `t_q`, which is a fraction of the peripheral clock
/// - the number of time quanta in a bit time, determined by `phase_seg_1` and
///   `phase_seg_2`
/// The configurable ranges of the parameters depend on which timing is changed.
///
/// This struct expects *real* values, extra subtractions and additions expected
/// the HW register are handled within the MCAN HAL.
///
/// Default values are:
/// - swj: 0x4
/// - phase_seg_1: 0xB
/// - phase_seg_1: 0x4
///
/// Default time quanta in a bit time is 16 (phase_seg_1 + phase_seg_2 +
/// synchronization segment (1))
#[derive(Copy, Clone)]
pub struct BitTiming {
    /// Synchronization jump width
    pub sjw: u8,
    /// Propagation time and phase time before sample point
    pub phase_seg_1: u8,
    /// Time after sample point
    pub phase_seg_2: u8,
    /// The bitrate of the bus. This needs to be chosen so that the clock to the
    /// MCAN peripheral is divisible into time quanta such that the bit time
    /// determined by `phase_seg_1` and `phase_seg_2` is a whole number of time
    /// quanta.
    pub bitrate: HertzU32,
}

impl BitTiming {
    /// Create an instance
    ///
    /// Nominal bitrate value must be provided, all other settings come
    /// pre-populated with default values.
    pub fn new(bitrate: HertzU32) -> Self {
        Self {
            // Note: SWJ and {N,D}TSEG{1,2} defaults come from reset values
            sjw: 0x4,
            phase_seg_1: 0xB,
            phase_seg_2: 0x4,
            bitrate,
        }
    }
}

/// Timestamp counter configuration
#[derive(Copy, Clone)]
pub struct Timestamp {
    /// Counting mode of time stamp timer
    pub select: TimeStampSelect,
    /// Time stamp timer prescaler, bit times per tick
    /// Valid values are: 1 <= ts_prescale <= 16
    pub prescaler: u8,
}

impl Default for Timestamp {
    fn default() -> Self {
        Self {
            select: TimeStampSelect::ZERO,
            prescaler: 1,
        }
    }
}

/// Misconfigurations of [`BitTiming`].
#[derive(Debug)]
pub enum BitTimingError {
    /// SJW is outside the wrapped `RangeInclusive`
    SynchronizationJumpWidthOutOfRange(RangeInclusive<u32>),
    /// Phase segment 1 is outside the wrapped `RangeInclusive`
    PhaseSeg1OutOfRange(RangeInclusive<u32>),
    /// Phase segment 2 is outside the wrapped `RangeInclusive`
    PhaseSeg2OutOfRange(RangeInclusive<u32>),
    /// Total bit time quanta is outside the wrapped `RangeInclusive`
    BitTimeOutOfRange(RangeInclusive<u32>),
    /// Prescaler is outside the wrapped `RangeInclusive`
    PrescalerOutOfRange(RangeInclusive<u32>),
    /// No valid prescaler could be found
    ///
    /// The following requirement must be met:
    /// - `can_clock` must be divisible by `bitrate * bit_time_quanta`
    NoValidPrescaler {
        /// Provided peripheral clock
        can_clock: HertzU32,
        /// Bitrate requested in [`BitTiming`]
        bitrate: HertzU32,
        /// Time quanta per bit selected by [`BitTiming`]
        bit_time_quanta: u32,
    },
}

/// Valid values of a BitTiming struct
#[derive(Clone)]
pub(crate) struct BitTimingRanges {
    sjw: RangeInclusive<u32>,
    phase_seg_1: RangeInclusive<u32>,
    phase_seg_2: RangeInclusive<u32>,
    /// The bit time, in time quanta
    time_quanta_per_bit: RangeInclusive<u32>,
    prescaler: RangeInclusive<u32>,
}
pub(crate) const NOMINAL_BIT_TIMING_RANGES: BitTimingRanges = BitTimingRanges {
    sjw: 1..=128,
    phase_seg_1: 2..=256,
    phase_seg_2: 2..=128,
    time_quanta_per_bit: 5..=385,
    prescaler: 1..=512,
};
pub(crate) const DATA_BIT_TIMING_RANGES: BitTimingRanges = BitTimingRanges {
    sjw: 1..=16,
    phase_seg_1: 1..=32,
    phase_seg_2: 1..=16,
    time_quanta_per_bit: 3..=49,
    prescaler: 1..=32,
};

impl BitTiming {
    /// Returns the number of time quanta that make up one bit time, `t_bit /
    /// t_q`
    pub fn time_quanta_per_bit(&self) -> u32 {
        1 + u32::from(self.phase_seg_1) + u32::from(self.phase_seg_2)
    }

    fn check(&self, valid: &BitTimingRanges) -> Result<(), BitTimingError> {
        if !valid.sjw.contains(&self.sjw.into()) {
            Err(BitTimingError::SynchronizationJumpWidthOutOfRange(
                valid.sjw.clone(),
            ))
        } else if !valid.phase_seg_1.contains(&self.phase_seg_1.into()) {
            Err(BitTimingError::PhaseSeg1OutOfRange(
                valid.phase_seg_1.clone(),
            ))
        } else if !valid.phase_seg_2.contains(&self.phase_seg_2.into()) {
            Err(BitTimingError::PhaseSeg2OutOfRange(
                valid.phase_seg_2.clone(),
            ))
        } else if !valid
            .time_quanta_per_bit
            .contains(&self.time_quanta_per_bit())
        {
            Err(BitTimingError::BitTimeOutOfRange(
                valid.time_quanta_per_bit.clone(),
            ))
        } else {
            Ok(())
        }
    }

    pub(crate) fn prescaler(
        &self,
        f_can: HertzU32,
        valid: &BitTimingRanges,
    ) -> Result<u16, BitTimingError> {
        self.check(valid)?;
        let f_out = self.bitrate;
        let bit_time_quanta = self.time_quanta_per_bit();
        let f_q = f_out * bit_time_quanta;
        if let Some(0) = f_can.to_Hz().checked_rem(f_q.to_Hz()) {
            let prescaler = f_can / f_q;
            if !valid.prescaler.contains(&prescaler) {
                Err(BitTimingError::PrescalerOutOfRange(valid.prescaler.clone()))
            } else {
                Ok(prescaler as u16)
            }
        } else {
            Err(BitTimingError::NoValidPrescaler {
                can_clock: f_can,
                bitrate: f_out,
                bit_time_quanta,
            })
        }
    }
}

/// Enable/disable CAN-FD and related features
#[derive(Default, Copy, Clone)]
pub enum Mode {
    /// Classic mode with 8-bytes data. Reception of an FD frame is considered
    /// an error.
    #[default]
    Classic,
    /// Transmission and reception of CAN FD frames (with up to 64 bytes of
    /// data) is enabled. This does not prevent use of classic CAN frames.
    Fd {
        /// If `true`, FD frames can be transmitted with bit rate switching.
        /// Otherwise, attempts to transmit FD frames will return errors.
        ///
        /// Regardless of this setting, data phase timing still must be
        /// configured as *reception* of bit-rate-switched messages is still
        /// possible.
        allow_bit_rate_switching: bool,
        /// Bit timing parameters for the data phase of bit rate switched FD
        /// frames.
        data_phase_timing: BitTiming,
    },
}

impl CanConfig {
    /// Create an instance
    ///
    /// Nominal bitrate value must be provided, all other settings come
    /// pre-populated with default values.
    pub fn new(bitrate: HertzU32) -> Self {
        Self {
            mode: Default::default(),
            loopback: Default::default(),
            nominal_timing: BitTiming::new(bitrate),
            timestamp: Default::default(),
            rx_fifo_0: Default::default(),
            rx_fifo_1: Default::default(),
            tx: Default::default(),
        }
    }
}

/// Denotes a RX FIFO configuration
#[derive(Default, Copy, Clone)]
pub struct RxFifoConfig {
    /// FIFO mode
    pub mode: RxFifoMode,
    /// Denotes queue fullness required to trigger a corresponding interrupt
    ///
    /// Any value greater than 64 is interpreted as 64; 0 means that interrupt
    /// is disabled
    pub watermark: u8,
}

/// Mode of operation for the RX FIFO
#[derive(Default, Copy, Clone)]
pub struct RxFifoMode(RxFifoModeVariant);

impl RxFifoMode {
    /// Blocking mode
    ///
    /// When the RX FIFO is full, incoming messages are dropped until at least
    /// one message has been read out from the FIFO.
    pub fn blocking() -> Self {
        Self(RxFifoModeVariant::Blocking)
    }
    /// Overwriting mode
    ///
    /// When the RX FIFO is full, the oldest messsage will be deleted and a new
    /// message will take its place.
    ///
    /// # Safety
    /// For the RX FIFO running in this mode, MCAN *does NOT provide* any
    /// synchronization primitives that user can rely on in order to guarantee
    /// integrity of the data being received.
    ///
    /// General guideline from the datasheet suggests that user should never
    /// read the oldest element in queue (as there is a risk that the message is
    /// currently being overwritten) and index should be offsetted by 1 or
    /// more (counting from the oldest message) depending on the speed of the
    /// CPU.
    ///
    /// Thus, it is up to the application developer to provide such an
    /// index offset so read out messages are correct.
    pub unsafe fn overwrite() -> Self {
        Self(RxFifoModeVariant::Overwrite)
    }
}

/// Mode of operation for the RX FIFO (inner enum)
#[derive(Default, Copy, Clone)]
pub enum RxFifoModeVariant {
    /// Blocking mode
    ///
    /// More details at [`RxFifoMode::blocking`]
    #[default]
    Blocking,
    /// Overwriting mode
    ///
    /// More details at [`RxFifoMode::overwrite`]
    Overwrite,
}

impl From<RxFifoMode> for bool {
    fn from(val: RxFifoMode) -> Self {
        match val.0 {
            RxFifoModeVariant::Overwrite => true,
            RxFifoModeVariant::Blocking => false,
        }
    }
}

impl From<RxFifoMode> for RxFifoModeVariant {
    fn from(val: RxFifoMode) -> Self {
        val.0
    }
}

/// Mode of operation for the transmit queue
#[derive(Default, Copy, Clone)]
pub enum TxQueueMode {
    /// Messages are sent according to the order they are enqueued
    #[default]
    Fifo,
    /// Messages are sent according to their priority
    ///
    /// Lower ID means higher priority. Messages of the same ID are sent in an
    /// arbitrary order. This is the same order as arbitration on the bus would
    /// give.
    Priority,
}

impl From<TxQueueMode> for bool {
    fn from(val: TxQueueMode) -> Self {
        match val {
            TxQueueMode::Priority => true,
            TxQueueMode::Fifo => false,
        }
    }
}
