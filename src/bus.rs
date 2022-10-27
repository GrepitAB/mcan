//! Pad declarations for the CAN buses

use crate::filter::Filters;
use crate::messageram::SharedMemoryInner;
use crate::reg::{ecr::R as ECR, psr::R as PSR};
use crate::rx_dedicated_buffers::RxDedicatedBuffer;
use crate::rx_fifo::{Fifo0, Fifo1, RxFifo};
use crate::tx_buffers::Tx;
use crate::tx_event_fifo::TxEventFifo;
use core::convert::From;
use core::fmt::{self, Debug};

use super::{
    config::{
        bus::{CanConfig, CanFdMode, InterruptConfiguration, TestMode},
        RamConfig,
    },
    message::{self, AnyMessage},
    messageram::{Capacities, SharedMemory},
};
use fugit::{HertzU32, RateExtU32};
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

/// Errors that may occur in the CAN bus
#[derive(Debug)]
pub enum Error {
    /// The bus needs to be in initialization mode, but is not
    NotInitializing,
    /// Specified mask contains one or more invalid tx buffer indices
    InvalidTxBuffer(u32),
    /// Buffer index is not valid for any new data field
    InvalidBufferIndex,
    /// The buffer has no new data
    BufferDataNotNew,
    /// Divider is too large for the peripheral
    InvalidDivider(f32),
    /// Generating the bitrate would require a division by less than one
    ZeroDivider,
    /// Input clock / divider yields a poor integer division
    BitTimeRounding(HertzU32),
    /// Output clock is not supposed to be running
    StoppedOutputClock,
    /// The bit-time quanta is zero
    ZeroQuanta,
    /// Divider is f32 NaN
    DividerIsNaN,
    /// Divider is f32 Inf
    DividerIsInf,
    /// Specified filter is invalid
    InvalidFilter,
    /// Indexed offset is out of bounds
    OutOfBounds,
    /// Targeted FIFO did not have any data to output
    FifoEmpty,
    /// Targeted FIFO is full and more elements can't be added
    FifoFull,
    /// Element capcity was larger than container type
    ElementOverflow,
    /// Errors from message construction
    MessageError(message::Error),
    /// Time stamp prescaler value is not in the range [1, 16]
    InvalidTimeStampPrescaler,
    /// The provided memory is not addressable by the peripheral.
    MemoryNotAddressable,
}

/// Index is out of bounds
pub struct OutOfBounds;

impl From<OutOfBounds> for Error {
    fn from(_: OutOfBounds) -> Self {
        Self::OutOfBounds
    }
}

impl From<message::Error> for Error {
    fn from(err: message::Error) -> Self {
        Self::MessageError(err)
    }
}

/// CAN bus results
pub type Result<T> = core::result::Result<T, Error>;

/// Token for identifying bus during runtime
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BusSlot {
    /// Peripheral 0
    Can0,
    /// Peripheral 1
    Can1,
}

/// Common CANbus functionality
/// TODO: build interrupt struct around this
pub trait CanBus {
    /// Read error counters
    fn error_counters(&self) -> ErrorCounters;
    /// Read additional status information
    fn protocol_status(&self) -> ProtocolStatus;
    /// Enable/disable loopback mode
    fn loopback(&mut self, state: bool);
    /// Enable/disable CAN-FD mode
    fn fd(&mut self, state: bool);
    /// Enable can device configuration mode
    fn enter_config_mode(&mut self);
    /// Enable can device operational mode
    fn enter_operational_mode(&mut self);
    /// Get current time
    fn ts_count(&self) -> u16;
}

/// A CAN bus
pub struct Can<'a, Id, D, C: Capacities> {
    /// Controls enabling and line selection of interrupts.
    pub interrupts: InterruptConfiguration<Id>,
    pub rx_fifo_0: RxFifo<'a, Fifo0, Id, C::RxFifo0Message>,
    pub rx_fifo_1: RxFifo<'a, Fifo1, Id, C::RxFifo1Message>,
    pub rx_dedicated_buffers: RxDedicatedBuffer<'a, Id, C::RxBufferMessage>,
    pub tx: Tx<'a, Id, C>,
    pub tx_event_fifo: TxEventFifo<'a, Id>,
    pub filters: Filters<'a, Id>,

    /// Implementation details. The field is public to allow destructuring.
    pub internals: Internals<Id, D>,
}

/// Implementation details.
pub struct Internals<Id, D> {
    /// CAN bus peripheral
    can: crate::reg::Can<Id>,
    dependencies: D,
}

impl<Id: crate::CanId, D: crate::Dependencies<Id>, C: Capacities> Can<'_, Id, D, C> {
    /// Raw access to the registers.
    ///
    /// # Safety
    /// The abstraction assumes that it has exclusive ownership of the
    /// registers. Direct access can break such assumptions. Direct access
    /// can break assumptions
    pub unsafe fn registers(&self) -> &crate::reg::Can<Id> {
        &self.internals.can
    }

    /// Apply parameters from a bus config struct
    /// Safety: Config may only be applied safely if the bus is initializing,
    ///         if the bus is not initializing, an error is returned
    fn apply_bus_config(&mut self, config: &CanConfig, freq: HertzU32) -> Result<()> {
        if !(1..=16).contains(&config.timing.ts_prescale) {
            return Err(Error::InvalidTimeStampPrescaler);
        }

        if !self.internals.can.cccr.read().init().bit() {
            return Err(Error::NotInitializing);
        } else {
            // Baud rate
            // TODO: rewrite this somewhat when we're required to implement variable data
            // rate!
            let c = self.internals.dependencies.can_clock().to_Hz();
            let f = freq.to_Hz();
            let q = config.timing.quanta();

            // Sanity check input parameters
            if f == 0 {
                return Err(Error::StoppedOutputClock);
            } else if q == 0 {
                return Err(Error::ZeroQuanta);
            }

            // Calculate divider
            let cf: f32 = c as f32;
            let ff: f32 = f as f32;
            let qf: f32 = q as f32;
            let divider = cf / (ff * qf);

            // Convert divider to u32
            // Safety: criterion and tested above, the divider is:
            //   * Not `NaN`
            //   * Not `Inf`
            //   * Divider smaller than 512 implies that it is smaller than max u32 and
            //     since it is generated from non-negative inputs it should be in range for
            //     u32
            let divider: u32 = if divider.is_nan() {
                return Err(Error::DividerIsNaN);
            } else if divider.is_infinite() {
                return Err(Error::DividerIsInf);
            } else if divider < 1.0f32 {
                // Dividers of < 1 round down to 0
                return Err(Error::ZeroDivider);
            } else if divider >= 512.0f32 {
                return Err(Error::InvalidDivider(divider));
            } else {
                unsafe { f32::to_int_unchecked(divider) }
            };

            // Compare the real output to the expected output
            let real_output = c / (divider * q as u32);

            if real_output != f {
                return Err(Error::BitTimeRounding(real_output.Hz()));
            } else {
                unsafe {
                    self.internals.can.nbtp.write(|w| {
                        w.nsjw()
                            .bits(config.timing.sjw)
                            .ntseg1()
                            .bits(config.timing.phase_seg_1)
                            .ntseg2()
                            .bits(config.timing.phase_seg_2)
                            .nbrp()
                            .bits((divider - 1) as u16)
                    });

                    self.internals.can.tscc.write(|w| {
                        w.tss()
                            .bits(config.timing.ts_select.into())
                            // Prescaler is 1 + tcp value.
                            .tcp()
                            .bits(config.timing.ts_prescale - 1)
                    });

                    // CAN-FD operation
                    self.internals
                        .can
                        .cccr
                        .modify(|_, w| w.fdoe().bit(config.fd_mode.clone().into()));
                    // HACK: Data bitrate is 1Mb/s
                    self.internals.can.dbtp.modify(|_, w| w.dbrp().bits(2));
                    self.internals
                        .can
                        .cccr
                        .modify(|_, w| w.brse().bit(config.bit_rate_switching));
                    // Global filter options
                    self.internals.can.gfc.write(|w| {
                        w.anfs()
                            .bits(config.nm_std.clone().into())
                            .anfe()
                            .bits(config.nm_ext.clone().into())
                    });

                    // Configure test/loopback mode
                    self.set_test(config.test.clone());
                }

                Ok(())
            }
        }
    }

    /// Apply parameters from a ram config struct
    ///
    /// Ensuring that the RAM config struct is properly defined is basically our
    /// only safeguard keeping the bus operational. Apart from that, the
    /// memory RAM is largely unchecked and an improperly configured linker
    /// script could interfere with bus operations.
    ///
    /// Safety: In order to run the CAN bus properly, the user must ensure that
    /// the         regions and parameters specified in `config` point to
    /// memory that is         not reserverd by another part of the
    /// application. Furthermore, the Config         may only be applied
    /// safely if the bus is initializing, if the bus is not
    ///         initializing, an error is returned.
    fn apply_ram_config(
        can: &crate::reg::Can<Id>,
        mem: &SharedMemoryInner<C>,
        config: &RamConfig,
    ) -> Result<()> {
        if !can.cccr.read().init().bit() {
            return Err(Error::NotInitializing);
        } else {
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
                    w.fom()
                        .bit(config.rxf0.mode.clone().into())
                        .fwm()
                        .bits(config.rxf0.watermark)
                        .fs()
                        .bits(mem.rx_fifo_0.len() as u8)
                        .fsa()
                        .bits(&mem.rx_fifo_0 as *const _ as u16)
                });

                //// RX FIFO 1
                can.rxf1.c.write(|w| {
                    w.fom()
                        .bit(config.rxf1.mode.clone().into())
                        .fwm()
                        .bits(config.rxf1.watermark)
                        .fs()
                        .bits(mem.rx_fifo_1.len() as u8)
                        .fsa()
                        .bits(&mem.rx_fifo_1 as *const _ as u16)
                });

                // TX buffers
                can.txbc.write(|w| {
                    w.tfqm()
                        .bit(config.txbc.mode.clone().into())
                        .tfqs()
                        .bits(
                            <C::TxBuffers as Unsigned>::U8
                                - <C::DedicatedTxBuffers as Unsigned>::U8,
                        )
                        .ndtb()
                        .bits(<C::DedicatedTxBuffers as Unsigned>::U8)
                        .tbsa()
                        .bits(&mem.tx_buffers as *const _ as u16)
                });

                // TX element size config
                can.txesc.write(|w| w.tbds().bits(C::TxMessage::REG));

                // TX events
                can.txefc.write(|w| {
                    w.efwm()
                        .bits(config.txefc.watermark)
                        .efs()
                        .bits(mem.tx_event_fifo.len() as u8)
                        .efsa()
                        .bits(&mem.tx_event_fifo as *const _ as u16)
                });
            }
            Ok(())
        }
    }

    /// Configure test mode
    fn set_fd(&mut self, fd: CanFdMode) {
        self.internals
            .can
            .cccr
            .modify(|_, w| w.fdoe().bit(fd.clone().into()));
    }

    /// Configure test mode
    fn set_test(&mut self, test: TestMode) {
        match test {
            TestMode::Disabled => {
                self.internals.can.cccr.modify(|_, w| w.test().bit(false));
                self.internals.can.test.modify(|_, w| w.lbck().bit(false));
            }
            TestMode::Loopback => {
                self.internals.can.cccr.modify(|_, w| w.test().bit(true));
                self.internals.can.test.modify(|_, w| w.lbck().bit(true));
            }
        }
    }
}

impl<Id: crate::CanId, D: crate::Dependencies<Id>, C: Capacities> CanBus for Can<'_, Id, D, C> {
    fn error_counters(&self) -> ErrorCounters {
        self.internals.can.ecr.read().into()
    }

    fn protocol_status(&self) -> ProtocolStatus {
        self.internals.can.psr.read().into()
    }

    /// Set CAN FD mode
    fn fd(&mut self, state: bool) {
        self.enter_config_mode();

        // Enable configuration change
        self.internals.can.cccr.modify(|_, w| w.cce().set_bit());
        while !self.internals.can.cccr.read().cce().bit() {
            // TODO: Make sure this loop does not get optimized away
        }

        // Configure CAN FD support
        if state {
            self.set_fd(CanFdMode::Fd);
        } else {
            self.set_fd(CanFdMode::Classic);
        }

        self.enter_operational_mode();
    }

    fn loopback(&mut self, state: bool) {
        self.enter_config_mode();

        // Enable configuration change
        self.internals.can.cccr.modify(|_, w| w.cce().set_bit());
        while !self.internals.can.cccr.read().cce().bit() {
            // TODO: Make sure this loop does not get optimized away
        }

        // Configure test/loopback mode
        if state {
            self.set_test(TestMode::Loopback);
        } else {
            self.set_test(TestMode::Disabled);
        }

        self.enter_operational_mode();
    }

    /// Enter configuration mode
    fn enter_config_mode(&mut self) {
        self.internals.can.cccr.modify(|_, w| w.init().set_bit());
        while !self.internals.can.cccr.read().init().bit() {
            // TODO: Make sure this loop does not get optimized away
        }
    }

    /// Enter operational mode
    fn enter_operational_mode(&mut self) {
        // Finish initializing peripheral
        self.internals.can.cccr.modify(|_, w| w.init().clear_bit());
        while self.internals.can.cccr.read().init().bit() {
            // TODO: Make sure this loop does not get optimized away
        }
    }

    fn ts_count(&self) -> u16 {
        self.internals.can.tscv.read().tsc().bits()
    }
}

impl<'a, Id: crate::CanId, D: crate::Dependencies<Id>, C: Capacities> Can<'a, Id, D, C> {
    /// Create new can peripheral.
    ///
    /// The hardware requires that SharedMemory is contained within the first
    /// 64K of system RAM. If this condition is not fulfilled, an error is
    /// returned.
    pub fn new(
        dependencies: D,
        freq: HertzU32,
        can_cfg: CanConfig,
        ram_cfg: RamConfig,
        memory: &'a mut SharedMemory<C>,
    ) -> Result<Self> {
        if !memory.is_addressable() {
            return Err(Error::MemoryNotAddressable);
        }

        // Safety:
        // Since `dependencies` field implies ownership of the HW register pointed to by
        // `Id: CanId`, `can` has a unique access to it
        let can = unsafe { crate::reg::Can::<Id>::new() };

        let memory = memory.init();
        Self::apply_ram_config(&can, memory, &ram_cfg).unwrap();

        let mut bus = Self {
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
            // Safety: The memory is zeroed by `memory.init`, so all filters are initially disabled.
            filters: unsafe {
                Filters::new(&mut memory.filters_standard, &mut memory.filters_extended)
            },
            internals: Internals { can, dependencies },
        };

        bus.enter_config_mode();

        // Enable configuration change
        bus.internals.can.cccr.modify(|_, w| w.cce().set_bit());
        while !bus.internals.can.cccr.read().cce().bit() {
            // TODO: Make sure this loop does not get optimized away
        }

        // Apply additional CAN config
        bus.apply_bus_config(&can_cfg, freq).unwrap();

        bus.enter_operational_mode();

        Ok(bus)
    }
}
