//! CAN bus configuration

pub use crate::reg::{self, tscc::TSS_A as TimeStampSelect};

/// Configuration for the CAN bus
pub struct CanConfig {
    /// TODO
    pub bit_rate_switching: bool,
    /// Run peripheral in CAN-FD mode
    pub fd_mode: CanFdMode,
    /// Modes of testing
    pub test: TestMode,
    /// Bit timing parameters
    pub timing: TimingParams,
    /// Action when handling non-matching standard frame
    pub nm_std: NonMatchingAction,
    /// Action when handling non-matching extended frame
    pub nm_ext: NonMatchingAction,
}

/// Bit-timing parameters
pub struct TimingParams {
    /// Synchronization jump width
    pub sjw: u8,
    /// Propagation time and phase time before sample point
    pub phase_seg_1: u8,
    /// Time after sample point
    pub phase_seg_2: u8,
    /// Counting mode of time stamp timer
    pub ts_select: TimeStampSelect,
    /// Time stamp timer prescaler, bittimes per tic
    /// Valid values are: 1 <= ts_prescale <= 16
    pub ts_prescale: u8,
}

impl TimingParams {
    /// Create a parameter field from spec-adherent values
    pub fn new(sjw: u8, phase_seg_1: u8, phase_seg_2: u8) -> Self {
        assert!(sjw < 128, "sjw > 127");
        assert!(phase_seg_1 > 0, "seg1 == 0");
        assert!(phase_seg_2 < 128, "seg2 > 127");

        Self {
            sjw,
            phase_seg_1,
            phase_seg_2,
            ts_select: TimeStampSelect::ZERO,
            ts_prescale: 1,
        }
    }

    /// Get total time for quanta
    pub fn quanta(&self) -> u16 {
        1 + ((self.phase_seg_1 as u16) + 1) + ((self.phase_seg_2 as u16) + 1)
    }
}

/// What to do with non-matching frames
#[derive(Clone)]
pub enum NonMatchingAction {
    /// Put frame in FIFO 0
    Fifo0,
    /// Put frame in FIFO 1
    Fifo1,
    /// Reject frame
    Reject,
}

impl Into<u8> for NonMatchingAction {
    fn into(self) -> u8 {
        match self {
            Self::Fifo0 => 0,
            Self::Fifo1 => 1,
            Self::Reject => 2,
        }
    }
}

impl Default for NonMatchingAction {
    fn default() -> Self {
        Self::Reject
    }
}

/// Enable/disable CAN-FD on the controller
#[derive(Clone)]
pub enum CanFdMode {
    /// Classic mode with 8-bytes data
    Classic,
    /// FD-mode with at most 64-bytes data
    Fd,
}

impl Into<bool> for CanFdMode {
    fn into(self) -> bool {
        match self {
            Self::Classic => false,
            Self::Fd => true,
        }
    }
}

/// Test modes for the bus
#[derive(Clone)]
pub enum TestMode {
    /// Do not initialize a test
    Disabled,
    /// Setup loopback
    Loopback,
}
