//! Pad declarations for the CAN buses

use crate::config::{BitTimingError, DATA_BIT_TIMING_RANGES, NOMINAL_BIT_TIMING_RANGES};
use crate::filter::{FiltersExtended, FiltersStandard};
use crate::interrupt::InterruptConfiguration;
use crate::messageram::SharedMemoryInner;
use crate::reg::{ecr::R as ECR, psr::R as PSR};
use crate::rx_dedicated_buffers::RxDedicatedBuffer;
use crate::rx_fifo::{Fifo0, Fifo1, RxFifo};
use crate::tx_buffers::Tx;
use crate::tx_event_fifo::TxEventFifo;
use core::convert::From;
use core::fmt::{self, Debug};
use core::ops::Deref;

use super::{
    config::{CanConfig, Mode},
    message::AnyMessage,
    messageram::{Capacities, SharedMemory},
};
use fugit::HertzU32;
use generic_array::typenum::Unsigned;

/// Wrapper for the protocol status register
pub struct ProtocolStatus(PSR);

impl Deref for ProtocolStatus {
    type Target = PSR;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<PSR> for ProtocolStatus {
    fn from(value: PSR) -> Self {
        Self(value)
    }
}

impl Debug for ProtocolStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProtocolStatus")
            .field("tdcv", &self.tdcv().bits())
            .field("pxe", &self.pxe().bits())
            .field("rfdf", &self.rfdf().bits())
            .field("rbrs", &self.rbrs().bits())
            .field("resi", &self.resi().bits())
            .field("dlec", &self.dlec().bits())
            .field("bo", &self.bo().bits())
            .field("ew", &self.ew().bits())
            .field("ep", &self.ep().bits())
            .field("act", &self.act().bits())
            .field("lec", &self.lec().bits())
            .finish()
    }
}

/// Wrapper for the error counters register
pub struct ErrorCounters(ECR);

impl Deref for ErrorCounters {
    type Target = ECR;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<ECR> for ErrorCounters {
    fn from(value: ECR) -> Self {
        Self(value)
    }
}

impl Debug for ErrorCounters {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ErrorCounters")
            .field("cel", &self.cel().bits())
            .field("rec", &self.rec().bits())
            .field("rp", &self.rp().bit())
            .field("tec", &self.tec().bits())
            .finish()
    }
}

/// Errors that may occur during configuration
#[derive(Debug)]
pub enum ConfigurationError {
    /// Problems with the bit timing configuration
    BitTiming(BitTimingError),
    /// Time stamp prescaler value is not in the range [1, 16]
    InvalidTimeStampPrescaler,
}

/// Error that may occur during construction
#[derive(Debug)]
pub struct MemoryNotAddressableError;

impl From<BitTimingError> for ConfigurationError {
    fn from(value: BitTimingError) -> Self {
        Self::BitTiming(value)
    }
}

/// A CAN bus that is not in configuration mode (CCE=0). Some errors (including
/// Bus_Off) can asynchronously stop bus operation (INIT=1), which will require
/// user intervention to reactivate the bus to resume sending and receiving
/// messages.
pub struct Can<'a, Id, D, C: Capacities> {
    /// Controls enabling and line selection of interrupts.
    pub interrupts: InterruptConfiguration<Id>,
    pub rx_fifo_0: RxFifo<'a, Fifo0, Id, C::RxFifo0Message>,
    pub rx_fifo_1: RxFifo<'a, Fifo1, Id, C::RxFifo1Message>,
    pub rx_dedicated_buffers: RxDedicatedBuffer<'a, Id, C::RxBufferMessage>,
    pub tx: Tx<'a, Id, C>,
    pub tx_event_fifo: TxEventFifo<'a, Id>,
    pub aux: Aux<'a, Id, D>,
}

/// Auxiliary struct
///
/// Provides unsafe low-level register access as well as other common CAN APIs
pub struct Aux<'a, Id, D> {
    /// CAN bus peripheral
    reg: crate::reg::Can<Id>,
    dependencies: D,
    config: CanConfig,
    filters_standard: FiltersStandard<'a, Id>,
    filters_extended: FiltersExtended<'a, Id>,
}

/// Trait which erases generic parametrization for [`Aux`] type
pub trait DynAux {
    /// CAN identity type
    type Id;

    /// CAN dependencies type
    type Deps;

    /// Re-enters "Normal Operation" if in "Software Initialization" mode.
    /// In Software Initialization, messages are not received or transmitted.
    /// Configuration cannot be changed. In Normal Operation, messages can
    /// be transmitted and received.
    fn operational_mode(&self);

    /// Returns `true` if the peripheral is in "Normal Operation" mode.
    fn is_operational(&self) -> bool;

    /// Access the error counters register value
    fn error_counters(&self) -> ErrorCounters;

    /// Access the protocol status register value
    ///
    /// Reading the register clears fields: PXE, RFDF, RBRS, RESI, DLEC, LEC.
    fn protocol_status(&self) -> ProtocolStatus;

    /// Current value of the timestamp counter
    ///
    /// If timestamping is disabled, its value is zero.
    fn timestamp(&self) -> u16;
}

impl<'a, Id: mcan_core::CanId, D: mcan_core::Dependencies<Id>> Aux<'a, Id, D> {
    fn configuration_mode(&self) {
        self.reg.configuration_mode()
    }
}

impl<'a, Id: mcan_core::CanId, D: mcan_core::Dependencies<Id>> DynAux for Aux<'a, Id, D> {
    type Id = Id;
    type Deps = D;

    fn operational_mode(&self) {
        self.reg.operational_mode();
    }

    fn is_operational(&self) -> bool {
        self.reg.is_operational()
    }

    fn error_counters(&self) -> ErrorCounters {
        ErrorCounters(self.reg.ecr.read())
    }

    fn protocol_status(&self) -> ProtocolStatus {
        ProtocolStatus(self.reg.psr.read())
    }

    fn timestamp(&self) -> u16 {
        self.reg.tscv.read().tsc().bits()
    }
}

/// A CAN bus in configuration mode. Before messages can be sent and received,
/// it needs to be [`Self::finalize`]d.
pub struct CanConfigurable<'a, Id, D, C: Capacities>(
    /// The type invariant of CCE=0 is broken while this is wrapped.
    Can<'a, Id, D, C>,
);

impl<'a, Id: mcan_core::CanId, D: mcan_core::Dependencies<Id>, C: Capacities>
    CanConfigurable<'a, Id, D, C>
{
    /// Raw access to the registers.
    ///
    /// # Safety
    /// The abstraction assumes that it has exclusive ownership of the
    /// registers. Direct access can break such assumptions. Direct access
    /// can break assumptions
    pub unsafe fn registers(&self) -> &crate::reg::Can<Id> {
        self.0.registers()
    }

    /// Allows reconfiguring the acceptance filters for standard IDs.
    pub fn filters_standard(&mut self) -> &mut FiltersStandard<'a, Id> {
        &mut self.0.aux.filters_standard
    }

    /// Allows reconfiguring the acceptance filters for extended IDs.
    pub fn filters_extended(&mut self) -> &mut FiltersExtended<'a, Id> {
        &mut self.0.aux.filters_extended
    }

    /// Allows reconfiguring interrupts.
    pub fn interrupts(&mut self) -> &mut InterruptConfiguration<Id> {
        &mut self.0.interrupts
    }

    /// Allows reconfiguring config
    pub fn config(&mut self) -> &mut CanConfig {
        &mut self.0.aux.config
    }

    /// Apply parameters from a bus config struct
    fn apply_configuration(&mut self) -> Result<(), ConfigurationError> {
        let reg = &self.0.aux.reg;
        let config = &self.0.aux.config;
        let dependencies = &self.0.aux.dependencies;
        if !(1..=16).contains(&config.timestamp.prescaler) {
            return Err(ConfigurationError::InvalidTimeStampPrescaler);
        }

        let nominal_prescaler = config
            .nominal_timing
            .prescaler(dependencies.can_clock(), &NOMINAL_BIT_TIMING_RANGES)?;

        // Safety: The configuration is checked to be valid when computing the prescaler
        reg.nbtp.write(|w| unsafe {
            w.nsjw()
                .bits(config.nominal_timing.sjw - 1)
                .ntseg1()
                .bits(config.nominal_timing.phase_seg_1 - 1)
                .ntseg2()
                .bits(config.nominal_timing.phase_seg_2 - 1)
                .nbrp()
                .bits(nominal_prescaler - 1)
        });

        // Safety: Every bit pattern of TCP is valid.
        reg.tscc.write(|w| unsafe {
            w.tss()
                .variant(config.timestamp.select)
                // Prescaler is 1 + tcp value.
                .tcp()
                .bits(config.timestamp.prescaler - 1)
        });

        match config.mode {
            Mode::Classic => reg.cccr.modify(|_, w| w.fdoe().clear_bit()),
            Mode::Fd {
                allow_bit_rate_switching,
                data_phase_timing,
            } => {
                reg.cccr
                    .modify(|_, w| w.fdoe().set_bit().brse().bit(allow_bit_rate_switching));
                let data_prescaler = data_phase_timing
                    .prescaler(dependencies.can_clock(), &DATA_BIT_TIMING_RANGES)?;
                // Safety: The configuration is checked to be valid when computing the prescaler
                reg.dbtp.write(|w| unsafe {
                    w.dsjw()
                        .bits(data_phase_timing.sjw - 1)
                        .dtseg1()
                        .bits(data_phase_timing.phase_seg_1 - 1)
                        .dtseg2()
                        .bits(data_phase_timing.phase_seg_2 - 1)
                        .dbrp()
                        .bits((data_prescaler - 1) as u8)
                });
            }
        };

        // Global filter configuration
        // This setting is redundant and the same behaviour is achievable through main
        // filter API
        reg.gfc.write(|w| {
            w.anfs()
                .variant(crate::reg::gfc::ANFS_A::REJECT)
                .anfe()
                .variant(crate::reg::gfc::ANFE_A::REJECT)
        });

        // Configure test/loopback mode
        reg.cccr.modify(|_, w| w.test().bit(config.loopback));
        reg.test.modify(|_, w| w.lbck().bit(config.loopback));

        // Configure RX FIFO 0
        reg.rxf0.c.modify(|_, w| {
            let w = w.fom().bit(config.rx_fifo_0.mode.into());
            let mut watermark = config.rx_fifo_0.watermark;
            // According to the spec, any value >= 64 is interpreted as watermark disabled
            if watermark >= 64 {
                watermark = 64;
            }
            // Safety: The value is sanitized before the write
            unsafe { w.fwm().bits(watermark) }
        });

        // Configure RX FIFO 1
        reg.rxf1.c.modify(|_, w| {
            let w = w.fom().bit(config.rx_fifo_1.mode.into());
            let mut watermark = config.rx_fifo_1.watermark;
            // According to the spec, any value >= 64 is interpreted as watermark disabled
            if watermark >= 64 {
                watermark = 64;
            }
            // Safety: The value is sanitized before the write
            unsafe { w.fwm().bits(watermark) }
        });

        // Configure Tx Buffer
        reg.txbc
            .modify(|_, w| w.tfqm().bit(config.tx.tx_queue_submode.into()));

        // Configure Tx Event Fifo
        reg.txefc.modify(|_, w| {
            let mut watermark = config.tx.tx_event_fifo_watermark;
            // According to the spec, any value >= 32 is interpreted as watermark disabled
            if watermark >= 32 {
                watermark = 32;
            }
            // Safety: The value is sanitized before the write
            unsafe { w.efwm().bits(watermark) }
        });
        Ok(())
    }

    /// Apply parameters from a ram config struct
    ///
    /// Ensuring that the RAM config struct is properly defined is basically our
    /// only safeguard keeping the bus operational. Apart from that, the
    /// memory RAM is largely unchecked and an improperly configured linker
    /// script could interfere with bus operations.
    fn apply_ram_config(reg: &crate::reg::Can<Id>, mem: &SharedMemoryInner<C>) {
        // Standard id
        //
        // Safety:
        // - Pointer is valid assuming SharedMemory location is within first 64K of RAM
        // - Length is checked at compile-time on the `Capacities` constraints level
        reg.sidfc.write(|w| unsafe {
            w.flssa()
                .bits(&mem.filters_standard as *const _ as u16)
                .lss()
                .bits(mem.filters_standard.len() as u8)
        });

        // Extended id
        //
        // Safety:
        // - Pointer is valid assuming SharedMemory location is within first 64K of RAM
        // - Length is checked at compile-time on the `Capacities` constraints level
        reg.xidfc.write(|w| unsafe {
            w.flesa()
                .bits(&mem.filters_extended as *const _ as u16)
                .lse()
                .bits(mem.filters_extended.len() as u8)
        });

        // RX buffers
        //
        // Safety:
        // - Pointer is valid assuming SharedMemory location is within first 64K of RAM
        reg.rxbc
            .write(|w| unsafe { w.rbsa().bits(&mem.rx_dedicated_buffers as *const _ as u16) });

        // Data field size for buffers and FIFOs
        reg.rxesc.write(|w| {
            w.rbds()
                .bits(C::RxBufferMessage::REG)
                .f0ds()
                .bits(C::RxFifo0Message::REG)
                .f1ds()
                .bits(C::RxFifo1Message::REG)
        });

        // RX FIFO 0
        //
        // Safety:
        // - Pointer is valid assuming SharedMemory location is within first 64K of RAM
        // - Length is checked at compile-time on the `Capacities` constraints level
        reg.rxf0.c.write(|w| unsafe {
            w.fsa()
                .bits(&mem.rx_fifo_0 as *const _ as u16)
                .fs()
                .bits(mem.rx_fifo_0.len() as u8)
        });

        // RX FIFO 1
        //
        // Safety:
        // - Pointer is valid assuming SharedMemory location is within first 64K of RAM
        // - Length is checked at compile-time on the `Capacities` constraints level
        reg.rxf1.c.write(|w| unsafe {
            w.fsa()
                .bits(&mem.rx_fifo_1 as *const _ as u16)
                .fs()
                .bits(mem.rx_fifo_1.len() as u8)
        });

        // TX buffers
        //
        // Safety:
        // - Pointer is valid assuming SharedMemory location is within first 64K of RAM
        // - Lengths are checked at compile-time on the `Capacities` constraints level
        reg.txbc.write(|w| unsafe {
            w.tfqs()
                .bits(<C::TxBuffers as Unsigned>::U8 - <C::DedicatedTxBuffers as Unsigned>::U8)
                .ndtb()
                .bits(<C::DedicatedTxBuffers as Unsigned>::U8)
                .tbsa()
                .bits(&mem.tx_buffers as *const _ as u16)
        });

        // TX element size config
        reg.txesc.write(|w| w.tbds().bits(C::TxMessage::REG));

        // TX events
        //
        // Safety:
        // - Pointer is valid assuming SharedMemory location is within first 64K of RAM
        // - Lengths are checked at compile-time on the `Capacities` constraints level
        reg.txefc.write(|w| unsafe {
            w.efsa()
                .bits(&mem.tx_event_fifo as *const _ as u16)
                .efs()
                .bits(mem.tx_event_fifo.len() as u8)
        });
    }

    /// Create new can peripheral.
    ///
    /// The hardware requires that SharedMemory is contained within the first
    /// 64K of system RAM. If this condition is not fulfilled, an error is
    /// returned.
    ///
    /// The returned peripheral is not operational; use [`Self::finalize`] to
    /// finish configuration and start transmitting and receiving.
    pub fn new(
        bitrate: HertzU32,
        dependencies: D,
        memory: &'a mut SharedMemory<C>,
    ) -> Result<Self, MemoryNotAddressableError> {
        // Safety:
        // Since `dependencies` field implies ownership of the HW register pointed to by
        // `Id: CanId`, `can` has a unique access to it
        let reg = unsafe { crate::reg::Can::<Id>::new() };

        reg.configuration_mode();

        // Contract:
        // `mcan_core::Dependencies::eligible_message_ram_start` contract guarantees
        // `u16::MAX + 1` alignment and points to the beginning of the allocatable CAN
        // memory region.
        if !memory.is_addressable(dependencies.eligible_message_ram_start()) {
            return Err(MemoryNotAddressableError);
        }

        let memory = memory.init();
        Self::apply_ram_config(&reg, memory);

        let config = CanConfig::new(bitrate);

        let can = CanConfigurable(Can {
            // Safety: Since `Can::new` takes a PAC singleton, it can only be called once. Then no
            // duplicates will be constructed. The registers that are delegated to these components
            // should not be touched by any other code. This has to be upheld by all code that has
            // access to the register block.
            interrupts: unsafe { InterruptConfiguration::new() },
            rx_fifo_0: unsafe { RxFifo::new(&mut memory.rx_fifo_0) },
            rx_fifo_1: unsafe { RxFifo::new(&mut memory.rx_fifo_1) },
            rx_dedicated_buffers: unsafe {
                RxDedicatedBuffer::new(&mut memory.rx_dedicated_buffers)
            },
            tx: unsafe { Tx::new(&mut memory.tx_buffers, config.mode) },
            tx_event_fifo: unsafe { TxEventFifo::new(&mut memory.tx_event_fifo) },
            aux: Aux {
                reg,
                dependencies,
                config,
                // Safety: The memory is zeroed by `memory.init`, so all filters are initially
                // disabled.
                filters_standard: unsafe { FiltersStandard::new(&mut memory.filters_standard) },
                filters_extended: unsafe { FiltersExtended::new(&mut memory.filters_extended) },
            },
        });

        Ok(can)
    }

    /// Locks the configuration and enters normal operation.
    pub fn finalize(mut self) -> Result<Can<'a, Id, D, C>, ConfigurationError> {
        self.apply_configuration()?;

        let can = self.0;

        // Enter normal operation (CCE is set to 0 automatically)
        can.aux.operational_mode();

        Ok(can)
    }
}

impl<'a, Id: mcan_core::CanId, D: mcan_core::Dependencies<Id>, C: Capacities> Can<'a, Id, D, C> {
    /// Raw access to the registers.
    ///
    /// # Safety
    /// The abstraction assumes that it has exclusive ownership of the
    /// registers. Direct access can break such assumptions.
    pub unsafe fn registers(&self) -> &crate::reg::Can<Id> {
        &self.aux.reg
    }

    pub fn configure(self) -> CanConfigurable<'a, Id, D, C> {
        self.aux.configuration_mode();
        CanConfigurable(self)
    }
}
