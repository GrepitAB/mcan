//! CAN bus configuration

pub use crate::reg::{self, tscc::TSS_A as TimeStampSelect};
use core::ops::RangeInclusive;
use fugit::HertzU32;

/// Configuration for the CAN bus
#[derive(Copy, Clone)]
pub struct CanConfig {
    /// Run peripheral in CAN-FD mode
    pub fd_mode: FdFeatures,
    /// Modes of testing
    pub test: TestMode,
    /// Bit timing parameters for everything except the data phase of bit rate
    /// switched FD frames.
    pub nominal_timing: BitTiming,
    /// Timestamp configuration
    pub timestamp: Timestamp,
    /// Action when handling non-matching standard frame
    pub nm_std: NonMatchingAction,
    /// Action when handling non-matching extended frame
    pub nm_ext: NonMatchingAction,
}

/// Bit-timing parameters. The bit time is determined by
/// - the time quantum `t_q`, which is a fraction of the peripheral clock
/// - the number of time quanta in a bit time, determined by `phase_seg_1` and
///   `phase_seg_2`
/// The configurable ranges of the parameters depend on which timing is changed.
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

#[derive(Copy, Clone)]
pub struct Timestamp {
    /// Counting mode of time stamp timer
    pub select: TimeStampSelect,
    /// Time stamp timer prescaler, bittimes per tic
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
    SynchronizationJumpWidth(RangeInclusive<u32>),
    PhaseSeg1(RangeInclusive<u32>),
    PhaseSeg2(RangeInclusive<u32>),
    BitTime(RangeInclusive<u32>),
    NoValidPrescaler,
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
    sjw: 0..=127,
    phase_seg_1: 1..=255,
    phase_seg_2: 1..=127,
    time_quanta_per_bit: 4..=385,
    prescaler: 0..=511,
};
pub(crate) const DATA_BIT_TIMING_RANGES: BitTimingRanges = BitTimingRanges {
    sjw: 0..=15,
    phase_seg_1: 0..=31,
    phase_seg_2: 0..=15,
    time_quanta_per_bit: 4..=49,
    prescaler: 0..=31,
};

impl BitTiming {
    /// Returns the number of time quanta that make up one bit time, `t_bit /
    /// t_q`
    pub fn time_quanta_per_bit(&self) -> u32 {
        1 + (u32::from(self.phase_seg_1) + 1) + (u32::from(self.phase_seg_2) + 1)
    }

    fn check(&self, valid: &BitTimingRanges) -> Result<(), BitTimingError> {
        if !valid.sjw.contains(&self.sjw.into()) {
            Err(BitTimingError::SynchronizationJumpWidth(valid.sjw.clone()))
        } else if !valid.phase_seg_1.contains(&self.phase_seg_1.into()) {
            Err(BitTimingError::PhaseSeg1(valid.phase_seg_1.clone()))
        } else if !valid.phase_seg_2.contains(&self.phase_seg_2.into()) {
            Err(BitTimingError::PhaseSeg2(valid.phase_seg_2.clone()))
        } else if !valid
            .time_quanta_per_bit
            .contains(&self.time_quanta_per_bit())
        {
            Err(BitTimingError::BitTime(valid.time_quanta_per_bit.clone()))
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
        let f_q = f_out * self.time_quanta_per_bit();
        if let Some(0) = f_can.to_Hz().checked_rem(f_q.to_Hz()) {
            let prescaler = f_can / f_q;
            if !valid.prescaler.contains(&prescaler) {
                Err(BitTimingError::NoValidPrescaler)
            } else {
                Ok(prescaler as u16)
            }
        } else {
            Err(BitTimingError::NoValidPrescaler)
        }
    }
}

/// What to do with non-matching frames
#[derive(Copy, Clone)]
pub enum NonMatchingAction {
    /// Put frame in FIFO 0
    Fifo0,
    /// Put frame in FIFO 1
    Fifo1,
    /// Reject frame
    Reject,
}

impl From<NonMatchingAction> for u8 {
    fn from(val: NonMatchingAction) -> Self {
        match val {
            NonMatchingAction::Fifo0 => 0,
            NonMatchingAction::Fifo1 => 1,
            NonMatchingAction::Reject => 2,
        }
    }
}

impl Default for NonMatchingAction {
    fn default() -> Self {
        Self::Reject
    }
}

/// Enable/disable CAN-FD and related features
#[derive(Copy, Clone)]
pub enum FdFeatures {
    /// Classic mode with 8-bytes data. Reception of an FD frame is considered
    /// an error.
    ClassicOnly,
    /// Transmission and reception of CAN FD frames (with up to 64 bytes of
    /// data) is enabled. This does not prevent use of classic CAN frames.
    Fd {
        /// If `true`, FD frames can be transmitted with bit rate switching.
        allow_bit_rate_switching: bool,
        /// Bit timing parameters for the data phase of bit rate switched FD
        /// frames.
        data_phase_timing: BitTiming,
    },
}

/// Test modes for the bus
#[derive(Copy, Clone)]
pub enum TestMode {
    /// Do not initialize a test
    Disabled,
    /// Setup loopback
    Loopback,
}
