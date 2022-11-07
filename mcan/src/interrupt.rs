use crate::reg;
use reg::AccessRegisterBlock as _;
use bitfield::bitfield;
use core::marker::PhantomData;

/// CAN interrupt lines
/// The CAN peripheral provides two interrupt lines to the system interrupt
/// controller. Which interrupts trigger which interrupt line is configurable
/// via [`InterruptConfiguration`].
#[derive(Copy, Clone)]
pub enum InterruptLine {
    /// CAN0-line
    Line0,
    /// CAN1-line
    Line1,
}

bitfield! {
    /// A set of CAN interrupts.
    #[derive(Copy, Clone)]
    pub struct InterruptSet(u32);

    /// Access to Reserved Address
    pub ara, set_ara:  29;
    /// Protocol Error in Data phase
    pub ped, set_ped:  28;
    /// Protocol Error in Arbitration phase
    pub pea, set_pea:  27;
    /// Watchdog
    pub wdi, set_wdi:  26;
    /// Bus Off
    pub bo, set_bo:   25;
    /// Warning status changed
    pub ew, set_ew:   24;
    /// Error Passive
    pub ep, set_ep:   23;
    /// Error Logging Overflow
    pub elo, set_elo:  22;
    /// Bit Error Uncorrected
    pub beu, set_beu:  21;
    /// Bit Error Corrected
    pub bec, set_bec:  20;
    /// Message stored to Dedicated Rx Buffer
    pub drx, set_drx:  19;
    /// Timeout Occured
    pub too, set_too:  18;
    /// Message Ram Access Failure
    pub mraf, set_mraf: 17;
    /// Timestamp Wraparound
    pub tsw, set_tsw:  16;
    /// Tx Event Fifo Element Lost
    pub tefl, set_tefl: 15;
    /// Tx Event Fifo Full
    pub teff, set_teff: 14;
    /// Tx Event Fifo Watermark Reached
    pub tefw, set_tefw: 13;
    /// Tx Event Fifo New Entry
    pub tefn, set_tefn: 12;
    /// Tx Fifo Empty
    pub tfe, set_tfe:  11;
    /// Transmission Cancellation Finished
    /// Note that there is a sub-interrupt for each transmit buffer element
    /// that is disabled by default. These sub-interrupts are enabled through
    /// [`Tx::enable_cancellation_interrupt`].
    pub tcf, set_tcf:  10;
    /// Transmission Completed
    /// Note that there is a sub-interrupt for each transmit buffer element
    /// that is disabled by default. These sub-interrupts are enabled through
    /// [`Tx::enable_transmission_completed_interrupt`].
    pub tc, set_tc:   9;
    /// High Priority Message
    pub hpm, set_hpm:  8;
    /// Rx Fifo1 Message Lost
    pub rf1l, set_rf1l: 7;
    /// Rx Fifo1 Full
    pub rf1f, set_rf1f: 6;
    /// Rx Fifo1 Watermark Reached
    pub rf1w, set_rf1w: 5;
    /// Rx Fifo1 New Message
    pub rf1n, set_rf1n: 4;
    /// Rx Fifo0 Message Lost
    pub rf0l, set_rf0l: 3;
    /// Rx Fifo0 Full
    pub rf0f, set_rf0f: 2;
    /// Rx Fifo0 Watermark Reached
    pub rf0w, set_rf0w: 1;
    /// Rx Fifo0 New Message
    pub rf0n, set_rf0n: 0;
}

impl FromIterator<Interrupt> for InterruptSet {
    fn from_iter<T: IntoIterator<Item = Interrupt>>(iter: T) -> Self {
        let mut set = 0_u32;
        for int in iter.into_iter() {
            set |= u32::from(int);
        }
        InterruptSet(set)
    }
}

impl core::fmt::Debug for InterruptSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "InterruptSet {{ ")?;
        if self.ara() {
            write!(f, "ARA ")?;
        }
        if self.ped() {
            write!(f, "PED ")?;
        }
        if self.pea() {
            write!(f, "PEA ")?;
        }
        if self.wdi() {
            write!(f, "WDI ")?;
        }
        if self.bo() {
            write!(f, "BO ")?;
        }
        if self.ew() {
            write!(f, "EW ")?;
        }
        if self.ep() {
            write!(f, "EP ")?;
        }
        if self.elo() {
            write!(f, "ELO ")?;
        }
        if self.beu() {
            write!(f, "BEU ")?;
        }
        if self.bec() {
            write!(f, "BEC ")?;
        }
        if self.drx() {
            write!(f, "DRX ")?;
        }
        if self.too() {
            write!(f, "TOO ")?;
        }
        if self.mraf() {
            write!(f, "MRAF ")?;
        }
        if self.tsw() {
            write!(f, "TSW ")?;
        }
        if self.tefl() {
            write!(f, "TEFL ")?;
        }
        if self.teff() {
            write!(f, "TEFF ")?;
        }
        if self.tefw() {
            write!(f, "TEFW ")?;
        }
        if self.tefn() {
            write!(f, "TEFN ")?;
        }
        if self.tfe() {
            write!(f, "TFE ")?;
        }
        if self.tcf() {
            write!(f, "TCF ")?;
        }
        if self.tc() {
            write!(f, "TC ")?;
        }
        if self.hpm() {
            write!(f, "HPM ")?;
        }
        if self.rf1l() {
            write!(f, "RF1L ")?;
        }
        if self.rf1f() {
            write!(f, "RF1F ")?;
        }
        if self.rf1w() {
            write!(f, "RF1W ")?;
        }
        if self.rf1n() {
            write!(f, "RF1N ")?;
        }
        if self.rf0l() {
            write!(f, "RF0L ")?;
        }
        if self.rf0f() {
            write!(f, "RF0F ")?;
        }
        if self.rf0w() {
            write!(f, "RF0W ")?;
        }
        if self.rf0n() {
            write!(f, "RF0N ")?;
        }
        write!(f, "}}")
    }
}

/// A single interrupt.
#[derive(Debug, Copy, Clone)]
pub enum Interrupt {
    /// RF0N
    RxFifo0NewMessage = 0,
    /// RF0W
    RxFifo0WatermarkReached = 1,
    /// RF0F
    RxFifo0Full = 2,
    /// RF0L
    RxFifo0MessageLost = 3,
    /// RF1N
    RxFifo1NewMessage = 4,
    /// RF1W
    RxFifo1WatermarkReached = 5,
    /// RF1F
    RxFifo1Full = 6,
    /// RF1L
    RxFifo1MessageLost = 7,
    /// HPM
    HighPriorityMessage = 8,
    /// TC
    TransmissionCompleted = 9,
    /// TCF
    TransmissionCancellationFinished = 10,
    /// TFE
    TxFifoEmpty = 11,
    /// TEFN
    TxEventFifoNewEntry = 12,
    /// TEFW
    TxEventFifoWatermarkReached = 13,
    /// TEFF
    TxEventFifoFull = 14,
    /// TEFL
    TxEventFifoElementLost = 15,
    /// TSW
    TimestampWraparound = 16,
    /// MRAF
    MessageRamAccessFailure = 17,
    /// TOO
    TimeoutOccured = 18,
    /// DRX
    MessageStoredToDedicatedRxBuffer = 19,
    /// BEC
    BitErrorCorrected = 20,
    /// BEU
    BitErrorUncorrected = 21,
    /// ELO
    ErrorLoggingOverflow = 22,
    /// EP
    ErrorPassive = 23,
    /// EW
    WarningStatusChanged = 24,
    /// BO
    BusOff = 25,
    /// WDI
    Watchdog = 26,
    /// PEA
    ProtocolErrorArbitration = 27,
    /// PED
    ProtocolErrorData = 28,
    /// ARA
    AccessToReservedAddress = 29,
}

impl From<Interrupt> for u32 {
    fn from(x: Interrupt) -> Self {
        1 << x as u32
    }
}

pub struct InvalidInterruptNumber;

impl TryFrom<u8> for Interrupt {
    type Error = InvalidInterruptNumber;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use Interrupt::*;
        let ret = match value {
            0 => RxFifo0NewMessage,
            1 => RxFifo0WatermarkReached,
            2 => RxFifo0Full,
            3 => RxFifo0MessageLost,
            4 => RxFifo1NewMessage,
            5 => RxFifo1WatermarkReached,
            6 => RxFifo1Full,
            7 => RxFifo1MessageLost,
            8 => HighPriorityMessage,
            9 => TransmissionCompleted,
            10 => TransmissionCancellationFinished,
            11 => TxFifoEmpty,
            12 => TxEventFifoNewEntry,
            13 => TxEventFifoWatermarkReached,
            14 => TxEventFifoFull,
            15 => TxEventFifoElementLost,
            16 => TimestampWraparound,
            17 => MessageRamAccessFailure,
            18 => TimeoutOccured,
            19 => MessageStoredToDedicatedRxBuffer,
            20 => BitErrorCorrected,
            21 => BitErrorUncorrected,
            22 => ErrorLoggingOverflow,
            23 => ErrorPassive,
            24 => WarningStatusChanged,
            25 => BusOff,
            26 => Watchdog,
            27 => ProtocolErrorArbitration,
            28 => ProtocolErrorData,
            29 => AccessToReservedAddress,
            30.. => Err(InvalidInterruptNumber)?,
        };
        Ok(ret)
    }
}

impl InterruptSet {
    /// An iterator visiting all elements in arbitrary order.
    pub fn iter(&self) -> Iter {
        Iter {
            flags: *self,
            index: 0,
        }
    }
}

/// An iterator over the items of an [`InterruptSet`].
///
/// This `struct` is created by [`InterruptSet::iter`].
pub struct Iter {
    flags: InterruptSet,
    index: u8,
}

impl Iterator for Iter {
    type Item = Interrupt;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.index;
        self.index = self.index.saturating_add(1);
        // Since there are no gaps in the interrupt flags, this will be `Some` until all
        // interrupts have been checked.
        let int = i.try_into().ok()?;
        if self.flags.0 & (1 << i) != 0 {
            Some(int)
        } else {
            self.next()
        }
    }
}

/// Has exclusive access to a set of interrupts for CAN peripheral `P`. Permits
/// safe access to the owned interrupt flags.
pub struct OwnedInterruptSet<P>(InterruptSet, PhantomData<P>);

/// An input [`InterruptSet`] contained interrupts that were not available. The
/// set wrapped in the error indicates which elements caused the problem.
#[derive(Debug)]
pub struct MaskError(pub InterruptSet);

impl<Id: mcan_core::CanId> OwnedInterruptSet<Id> {
    /// Assumes exclusive ownership of `interrupts`.
    ///
    /// # Safety
    /// Each interrupt of a CAN peripheral can only be contained in one
    /// `OwnedInterruptSet`, otherwise registers will be mutably aliased.
    ///
    /// The reserved bits must not be included.
    unsafe fn new(interrupts: InterruptSet) -> Self {
        Self(interrupts, PhantomData)
    }

    /// Moves ownership of the interrupts described by `subset` from `self` to
    /// the return value. If `self` does not contain `subset`, an error is
    /// returned.
    fn split(&mut self, subset: InterruptSet) -> Result<Self, MaskError> {
        let missing = !self.0 .0 & subset.0;
        if missing != 0 {
            Err(MaskError(InterruptSet(missing)))
        } else {
            let remaining = self.0 .0 & !subset.0;
            self.0 .0 = remaining;
            // Safety: No aliasing is introduced since `subset` is moved from `self`.
            unsafe { Ok(Self::new(subset)) }
        }
    }

    /// Assume ownership of the interrupts in `other`.
    fn join(&mut self, other: Self) {
        // The sets should be disjoint as long as the constructor is used safely.
        debug_assert!(self.0 .0 & other.0 .0 == 0);
        // No assurance is provided at this level that the sets are assigned to the same
        // interrupt line.
        self.0 .0 |= other.0 .0;
    }

    /// Clears the flagged interrupts owned by this `OwnedInterruptSet` and
    /// provides an iterator over the flags that were cleared.
    pub fn iter_flagged(&self) -> Iter {
        let interrupts = self.interrupt_flags();
        self.clear_interrupts(interrupts);
        interrupts.iter()
    }

    /// # Safety
    /// This gives access to reads and (through interior mutability) writes of
    /// IR. The bits not owned by this set must not be affected by these writes
    /// and must not be relied on by these reads.
    unsafe fn ir(&self) -> &reg::IR {
        &(*Id::register_block()).ir
    }

    /// Get the subset of interrupts in this set that are currently flagged.
    pub fn interrupt_flags(&self) -> InterruptSet {
        // Safety: The mask ensures that only flags under our control are returned.
        let masked = unsafe { self.ir().read().bits() & self.0 .0 };
        InterruptSet(masked)
    }

    /// Clear the indicated `interrupts`. Interrupts not owned by this
    /// `OwnedInterruptSet` are silently ignored.
    pub fn clear_interrupts(&self, interrupts: InterruptSet) {
        let masked = interrupts.0 & self.0 .0;
        // Safety: Writing a 0 bit leaves the flag unchanged, so masking the write with
        // the owned interrupts ensures no no other bits are affected. Reserved bits
        // will not be written because they will never be owned.
        unsafe {
            self.ir().write(|w| w.bits(masked));
        }
    }
}

/// Controls enabling and line selection of interrupts.
pub struct InterruptConfiguration<P> {
    disabled: OwnedInterruptSet<P>,
    _peripheral: PhantomData<P>,
}

impl<Id: mcan_core::CanId> InterruptConfiguration<Id> {
    /// # Safety
    /// This type takes ownership of some of the registers from the peripheral
    /// RegisterBlock. Do not use them to avoid aliasing. Do not instantiate
    /// more than once.
    /// - ILS
    /// - ILE
    /// - IE
    /// - IR
    pub(crate) unsafe fn new() -> Self {
        let v = Self {
            // Safety: this represents owning all of IR, which is ensured by the safety comment on
            // the constructor. The reserved bits are exluded.
            disabled: OwnedInterruptSet::new(InterruptSet(0x3fff_ffff)),
            _peripheral: PhantomData,
        };
        // Disable all interrupts on the peripheral by writing the reset value.
        v.ils().write(|w| w);
        v
    }

    fn ils(&self) -> &reg::ILS {
        // Safety: The constructor sets self up to have exclusive access to ILS.
        &unsafe { &*Id::register_block() }.ils
    }

    fn ile(&self) -> &reg::ILE {
        // Safety: The constructor sets self up to have exclusive access to ILE.
        &unsafe { &*Id::register_block() }.ile
    }

    fn ie(&self) -> &reg::IE {
        // Safety: The constructor sets self up to have exclusive access to IE.
        &unsafe { &*Id::register_block() }.ie
    }

    /// Request to enable the set of `interrupts` on the chosen interrupt line.
    /// Fails if some of the requested interrupts are already enabled.
    pub fn enable(
        &mut self,
        interrupts: InterruptSet,
        line: InterruptLine,
    ) -> Result<OwnedInterruptSet<Id>, MaskError> {
        let interrupts = self.disabled.split(interrupts)?;
        self.set_line(&interrupts, line);
        self.set_enabled(&interrupts, true);
        Ok(interrupts)
    }

    /// Disable the set of `interrupts` and move ownership back to the
    /// `InterruptConfiguration`.
    pub fn disable(&mut self, interrupts: OwnedInterruptSet<Id>) {
        self.set_enabled(&interrupts, false);
        self.disabled.join(interrupts);
    }

    /// Set the interrupt line that will trigger for a set of peripheral
    /// interrupts.
    pub fn set_line(&mut self, interrupts: &OwnedInterruptSet<Id>, line: InterruptLine) {
        self.enable_line(line);
        let mask = interrupts.0 .0;
        // Safety: The reserved bits are 0 by type invariant on `OwnedInterruptSet`.
        self.ils().modify(|r, w| unsafe {
            w.bits(match line {
                InterruptLine::Line0 => r.bits() & !mask,
                InterruptLine::Line1 => r.bits() | mask,
            })
        });
    }

    fn enable_line(&mut self, line: InterruptLine) {
        self.ile().modify(|_, w| match line {
            InterruptLine::Line0 => w.eint0().set_bit(),
            InterruptLine::Line1 => w.eint1().set_bit(),
        });
    }

    fn set_enabled(&mut self, interrupts: &OwnedInterruptSet<Id>, enabled: bool) {
        let mask = interrupts.0 .0;
        // Safety: The reserved bits are 0 by type invariant on `OwnedInterruptSet`.
        self.ie().modify(|r, w| unsafe {
            w.bits(if enabled {
                r.bits() | mask
            } else {
                r.bits() & !mask
            })
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iter_preserves_length() {
        assert_eq!(InterruptSet(0).iter().count(), 0);
        assert_eq!(InterruptSet(1).iter().count(), 1);
        assert_eq!(InterruptSet(0x1555_5555).iter().count(), 15);
        assert_eq!(InterruptSet(0x2aaa_aaaa).iter().count(), 15);
        assert_eq!(InterruptSet(0x3fff_ffff).iter().count(), 30);
        assert_eq!(InterruptSet(0xffff_ffff).iter().count(), 30);
    }

    fn iter_collect(int: u32) -> u32 {
        InterruptSet::from_iter(InterruptSet(int).iter()).0
    }

    #[test]
    fn iter_collect_preserves_interrupts() {
        assert_eq!(iter_collect(0), 0);
        assert_eq!(iter_collect(1), 1);
        assert_eq!(iter_collect(0x1555_5555), 0x1555_5555);
        assert_eq!(iter_collect(0x2aaa_aaaa), 0x2aaa_aaaa);
    }

    #[test]
    fn iter_collect_drops_reserved_bits() {
        assert_eq!(iter_collect(0xffff_ffff), 0x3fff_ffff);
    }
}
