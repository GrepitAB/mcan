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

use super::{
    config::{CanConfig, Mode},
    message::AnyMessage,
    messageram::{Capacities, SharedMemory},
};
use fugit::HertzU32;
use generic_array::typenum::Unsigned;

/// Printable PSR field
pub struct ProtocolStatus(pub PSR);

impl From<PSR> for ProtocolStatus {
    fn from(value: PSR) -> Self {
        Self(value)
    }
}

impl Debug for ProtocolStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> fmt::Result {
        let psr = &self.0;

        f.debug_struct("ProtocolStatus")
            .field("tdcv", &psr.tdcv().bits())
            .field("pxe", &psr.pxe().bits())
            .field("rfdf", &psr.rfdf().bits())
            .field("rbrs", &psr.rbrs().bits())
            .field("resi", &psr.resi().bits())
            .field("dlec", &psr.dlec().bits())
            .field("bo", &psr.bo().bits())
            .field("ew", &psr.ew().bits())
            .field("ep", &psr.ep().bits())
            .field("act", &psr.act().bits())
            .field("lec", &psr.lec().bits())
            .finish()
    }
}

/// Printable ECR field
pub struct ErrorCounters(pub ECR);

impl From<ECR> for ErrorCounters {
    fn from(value: ECR) -> Self {
        Self(value)
    }
}

impl Debug for ErrorCounters {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> fmt::Result {
        let ecr = &self.0;

        f.debug_struct("ErrorCounters")
            .field("cel", &ecr.cel().bits())
            .field("rec", &ecr.rec().bits())
            .field("rp", &ecr.rp().bit())
            .field("tec", &ecr.tec().bits())
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

/// Index is out of bounds
#[derive(Debug)]
pub struct OutOfBounds;

/// Common CANbus functionality
/// TODO: build interrupt struct around this
pub trait CanBus {
    /// Read error counters
    fn error_counters(&self) -> ErrorCounters;
    /// Read additional status information
    fn protocol_status(&self) -> ProtocolStatus;
    /// Get current time
    fn ts_count(&self) -> u16;
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

    /// Implementation details. The field is public to allow destructuring.
    pub internals: Internals<'a, Id, D>,
}

/// Implementation details.
pub struct Internals<'a, Id, D> {
    // TODO: Maybe it should be "reg" or sth, "can" feels very ambiguous
    /// CAN bus peripheral
    can: crate::reg::Can<Id>,
    dependencies: D,
    config: CanConfig,
    filters_standard: FiltersStandard<'a, Id>,
    filters_extended: FiltersExtended<'a, Id>,
}

impl<'a, Id: mcan_core::CanId, D: mcan_core::Dependencies<Id>> Internals<'a, Id, D> {
    // TODO: Maybe it should be unsafe?
    /// Switches between "Software Initialization" mode and "Normal Operation".
    /// In Software Initialization, messages are not received or transmitted.
    /// Configuration cannot be changed. In Normal Operation, messages can
    /// be transmitted and received.
    pub fn set_init(&mut self, init: bool) {
        self.can.cccr.modify(|_, w| w.init().bit(init));
        while self.can.cccr.read().init().bit() != init {}
    }

    fn enable_cce(&mut self) {
        self.can.cccr.modify(|_, w| w.cce().set_bit());
        while !self.can.cccr.read().cce().bit() {}
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
        &mut self.0.internals.filters_standard
    }

    /// Allows reconfiguring the acceptance filters for extended IDs.
    pub fn filters_extended(&mut self) -> &mut FiltersExtended<'a, Id> {
        &mut self.0.internals.filters_extended
    }

    /// Allows reconfiguring interrupts.
    pub fn interrupts(&mut self) -> &mut InterruptConfiguration<Id> {
        &mut self.0.interrupts
    }

    pub fn config(&mut self) -> &mut CanConfig {
        &mut self.0.internals.config
    }

    /// Apply parameters from a bus config struct
    fn apply_bus_config(&mut self) -> Result<(), ConfigurationError> {
        let can = &self.0.internals.can;
        let config = &self.0.internals.config;
        let dependencies = &self.0.internals.dependencies;
        if !(1..=16).contains(&config.timestamp.prescaler) {
            return Err(ConfigurationError::InvalidTimeStampPrescaler);
        }

        let nominal_prescaler = config
            .nominal_timing
            .prescaler(dependencies.can_clock(), &NOMINAL_BIT_TIMING_RANGES)?;

        // Safety: The configuration is checked to be valid when computing the prescaler
        can.nbtp.write(|w| unsafe {
            w.nsjw()
                .bits(config.nominal_timing.sjw)
                .ntseg1()
                .bits(config.nominal_timing.phase_seg_1)
                .ntseg2()
                .bits(config.nominal_timing.phase_seg_2)
                .nbrp()
                .bits(nominal_prescaler - 1)
        });

        // Safety: Every bit pattern of TCP is valid.
        can.tscc.write(|w| unsafe {
            w.tss()
                .variant(config.timestamp.select)
                // Prescaler is 1 + tcp value.
                .tcp()
                .bits(config.timestamp.prescaler - 1)
        });

        match config.mode {
            Mode::Classic => can.cccr.modify(|_, w| w.fdoe().clear_bit()),
            Mode::Fd {
                allow_bit_rate_switching,
                data_phase_timing,
            } => {
                can.cccr
                    .modify(|_, w| w.fdoe().set_bit().brse().bit(allow_bit_rate_switching));
                let data_divider = data_phase_timing
                    .prescaler(dependencies.can_clock(), &DATA_BIT_TIMING_RANGES)?;
                // Safety: The configuration is checked to be valid when computing the prescaler
                can.dbtp.write(|w| unsafe {
                    w.dsjw()
                        .bits(data_phase_timing.sjw)
                        .dtseg1()
                        .bits(data_phase_timing.phase_seg_1)
                        .dtseg2()
                        .bits(data_phase_timing.phase_seg_2)
                        .dbrp()
                        .bits((data_divider - 1) as u8)
                });
            }
        };

        // Global filter configuration
        // This setting is redundant and the same behaviour is achievable through main
        // filter API
        can.gfc.write(|w| {
            w.anfs()
                .variant(crate::reg::gfc::ANFS_A::REJECT)
                .anfe()
                .variant(crate::reg::gfc::ANFE_A::REJECT)
        });

        // Configure test/loopback mode
        can.cccr.modify(|_, w| w.test().bit(config.loopback));
        can.test.modify(|_, w| w.lbck().bit(config.loopback));

        // Configure RX FIFO 0
        can.rxf0.c.modify(|_, w| unsafe {
            w.fom()
                .bit(config.rx_fifo_0.mode.into())
                .fwm()
                .bits(config.rx_fifo_0.watermark)
        });

        // Configure RX FIFO 1
        can.rxf1.c.modify(|_, w| unsafe {
            w.fom()
                .bit(config.rx_fifo_1.mode.into())
                .fwm()
                .bits(config.rx_fifo_1.watermark)
        });

        // Configure Tx Buffer
        can.txbc
            .modify(|_, w| w.tfqm().bit(config.tx.tx_buffer_mode.into()));

        // Configure Tx Event Fifo
        can.txefc
            .modify(|_, w| unsafe { w.efwm().bits(config.tx.tx_event_fifo_watermark) });
        Ok(())
    }

    /// Apply parameters from a ram config struct
    ///
    /// Ensuring that the RAM config struct is properly defined is basically our
    /// only safeguard keeping the bus operational. Apart from that, the
    /// memory RAM is largely unchecked and an improperly configured linker
    /// script could interfere with bus operations.
    fn apply_ram_config(can: &crate::reg::Can<Id>, mem: &SharedMemoryInner<C>) {
        // TODO: Narrow down the unsafe usage?
        // TODO: Maybe move the HW calls into respective CAN subfields construction
        unsafe {
            // Standard id
            can.sidfc.write(|w| {
                w.flssa()
                    .bits(&mem.filters_standard as *const _ as u16)
                    .lss()
                    .bits(mem.filters_standard.len() as u8)
            });

            // Extended id
            can.xidfc.write(|w| {
                w.flesa()
                    .bits(&mem.filters_extended as *const _ as u16)
                    .lse()
                    .bits(mem.filters_extended.len() as u8)
            });

            // RX buffers
            can.rxbc
                .write(|w| w.rbsa().bits(&mem.rx_dedicated_buffers as *const _ as u16));

            // Data field size for buffers and FIFOs
            can.rxesc.write(|w| {
                w.rbds()
                    .bits(C::RxBufferMessage::REG)
                    .f0ds()
                    .bits(C::RxFifo0Message::REG)
                    .f1ds()
                    .bits(C::RxFifo1Message::REG)
            });

            //// RX FIFO 0
            can.rxf0.c.write(|w| {
                w.fs()
                    .bits(mem.rx_fifo_0.len() as u8)
                    .fsa()
                    .bits(&mem.rx_fifo_0 as *const _ as u16)
            });

            //// RX FIFO 1
            can.rxf1.c.write(|w| {
                w.fs()
                    .bits(mem.rx_fifo_1.len() as u8)
                    .fsa()
                    .bits(&mem.rx_fifo_1 as *const _ as u16)
            });

            // TX buffers
            can.txbc.write(|w| {
                w.tfqs()
                    .bits(<C::TxBuffers as Unsigned>::U8 - <C::DedicatedTxBuffers as Unsigned>::U8)
                    .ndtb()
                    .bits(<C::DedicatedTxBuffers as Unsigned>::U8)
                    .tbsa()
                    .bits(&mem.tx_buffers as *const _ as u16)
            });

            // TX element size config
            can.txesc.write(|w| w.tbds().bits(C::TxMessage::REG));

            // TX events
            can.txefc.write(|w| {
                w.efs()
                    .bits(mem.tx_event_fifo.len() as u8)
                    .efsa()
                    .bits(&mem.tx_event_fifo as *const _ as u16)
            });
        }
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
        let can = unsafe { crate::reg::Can::<Id>::new() };

        if !memory.is_addressable() {
            return Err(MemoryNotAddressableError);
        }

        let memory = memory.init();
        Self::apply_ram_config(&can, &memory);

        let can = Can {
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
            tx: unsafe { Tx::new(&mut memory.tx_buffers) },
            tx_event_fifo: unsafe { TxEventFifo::new(&mut memory.tx_event_fifo) },
            internals: Internals {
                can,
                dependencies,
                config: CanConfig::new(bitrate),
                // Safety: The memory is zeroed by `memory.init`, so all filters are initially
                // disabled.
                filters_standard: unsafe { FiltersStandard::new(&mut memory.filters_standard) },
                filters_extended: unsafe { FiltersExtended::new(&mut memory.filters_extended) },
            },
        }
        .configure();

        Ok(can)
    }

    /// Locks the configuration and enters normal operation.
    pub fn finalize(mut self) -> Result<Can<'a, Id, D, C>, ConfigurationError> {
        self.apply_bus_config()?;

        let mut can = self.0;
        // Enter normal operation (CCE is set to 0 automatically)
        can.internals.set_init(false);

        Ok(can)
    }
}

impl<'a, Id: mcan_core::CanId, D: mcan_core::Dependencies<Id>, C: Capacities> Can<'a, Id, D, C> {
    /// Raw access to the registers.
    ///
    /// # Safety
    /// The abstraction assumes that it has exclusive ownership of the
    /// registers. Direct access can break such assumptions. Direct access
    /// can break assumptions
    pub unsafe fn registers(&self) -> &crate::reg::Can<Id> {
        &self.internals.can
    }

    pub fn configure(mut self) -> CanConfigurable<'a, Id, D, C> {
        self.internals.set_init(true);
        self.internals.enable_cce();
        CanConfigurable(self)
    }
}

impl<Id: mcan_core::CanId, D: mcan_core::Dependencies<Id>, C: Capacities> CanBus
    for Can<'_, Id, D, C>
{
    fn error_counters(&self) -> ErrorCounters {
        self.internals.can.ecr.read().into()
    }

    fn protocol_status(&self) -> ProtocolStatus {
        self.internals.can.psr.read().into()
    }

    fn ts_count(&self) -> u16 {
        self.internals.can.tscv.read().tsc().bits()
    }
}
