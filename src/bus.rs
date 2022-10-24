//! Pad declarations for the CAN buses

use crate::reg::{ecr::R as ECR, psr::R as PSR};
use core::cmp::min;
use core::convert::From;
use core::fmt::{self, Debug};

use super::{
    config::{
        bus::{CanConfig, CanFdMode, InterruptConfiguration, TestMode},
        RamConfig,
    },
    filter::{ExtFilter, Filter},
    message::{self, rx, tx, AnyMessage},
    messageram::{Capacities, SharedMemory, SharedMemoryInner},
};
use embedded_hal::can::Id;
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

impl From<message::Error> for Error {
    fn from(err: message::Error) -> Self {
        Self::MessageError(err)
    }
}

/// CAN bus results
pub type Result<T> = core::result::Result<T, Error>;

/// RX fifo
pub trait CanFifo {
    /// Interrupt index offset
    const INT_OFFSET: usize;
}

/// FIFO 0 representation
pub struct Fifo0;

impl CanFifo for Fifo0 {
    const INT_OFFSET: usize = 0;
}

/// FIFO 0 representation
pub struct Fifo1;

impl CanFifo for Fifo1 {
    const INT_OFFSET: usize = 4;
}

/// Token for identifying bus during runtime
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BusSlot {
    /// Peripheral 0
    Can0,
    /// Peripheral 1
    Can1,
}

/// Setup message transmissions
pub trait CanSendBuffer<C: Capacities> {
    /// Transmit message with id
    fn transmit_buffer(&mut self, id: usize) -> Result<()>;

    /// Write TX message to transmission buffer
    ///
    /// ```text
    /// [ ID3_, ID15, XXXX, ____, ID8_, ID24 | ____, ID4_, ID2_, ____ ]
    ///               ^index
    /// ```
    fn send_buffer(&mut self, index: usize, message: C::TxMessage, auto_send: bool) -> Result<()>;
}

/// Slice output interface
pub trait CanSendSlice<C: Capacities> {
    /// Send a slice through tx fifo
    fn send_slice(&mut self, id: Id, data: &[u8]) -> Result<()>;
}

/// Read transmitted messages from FIFO
pub trait CanReadFifo<F: CanFifo, C: Capacities, M: rx::AnyMessage>
where
    Self: CanBus,
{
    /// Pop the topmost message off the FIFO and return it
    fn read(&mut self) -> Result<M> {
        // Get the top message
        let message = self.peek()?;

        // Pop the message
        Self::mark_fifo_read(self)?;

        // Clear RX interrupt if this is the last message
        if Self::fill(self) == 0 {
            <Self as CanBus>::clear_interrupts(self, 1 << F::INT_OFFSET);
        }

        // Return peeked message
        Ok(message)
    }

    /// Pop the topmost message off the FIFO and write its data to a slice
    fn read_slice(&mut self, output: &mut [u8]) -> Result<()> {
        let message: M = self.read()?;
        let data = message.data();

        if data.len() > output.len() {
            // The output slice is too small
            Err(Error::ElementOverflow)
        } else {
            output[..data.len()].copy_from_slice(data);
            Ok(())
        }
    }

    /// Get first message without popping it
    // TODO: implement as default in trait
    fn peek(&mut self) -> Result<M>;

    /// Mark an item as read
    fn mark_fifo_read(&mut self) -> Result<()>;

    /// Get the current get index
    fn get(&self) -> usize;

    /// Get current put index
    fn put(&self) -> usize;

    /// Get number of elements in buffer
    fn fill(&self) -> usize;

    /// Get number of free slots in buffer
    fn free(&self) -> usize;
}

/// Write transmission messages to FIFO
pub trait CanSendFifo<C: Capacities> {
    /// Mark an item as read
    fn transmit_fifo(&mut self, index: usize) -> Result<()>;

    /// Get the current put index
    fn put(&mut self) -> Result<usize>;

    /// Add a message to a tranmission FIFO
    ///
    /// ```text
    /// [ ID3_, ID15, ____, ____, ID8_, ID24 | ____, ID4_, ID2_, XXXX ]
    ///                                                          ^put
    /// ```
    fn send_fifo(&mut self, message: C::TxMessage) -> Result<()>;
}

/// Read transmitted messages from buffer
pub trait CanReadBuffer<C: Capacities> {
    /// Read a message in the message buffer
    ///
    /// ```text
    /// [ ID3_, ____, ____, ID8_, ID24 | ____, ID4_, ID2_, ____ ]
    ///         V index
    ///         ID15,
    /// ```
    fn read_buffer(&mut self, index: usize) -> Result<C::RxBufferMessage>;

    /// Mark an item as read
    fn mark_buffer_read(&mut self, index: usize) -> Result<()>;
}

/// Common CANbus functionality
/// TODO: build interrupt struct around this
pub trait CanBus {
    /// Read error counters
    fn error_counters(&self) -> ErrorCounters;
    /// Read additional status information
    fn protocol_status(&self) -> ProtocolStatus;
    /// Read new data status for message at index
    fn new_data(&self, index: usize) -> Result<bool>;
    /// Mask interrupts on
    fn enable_interrupts(&mut self, mask: u32);
    /// Mask interrupts off
    fn disable_interrupts(&mut self, mask: u32);
    /// Apply new interrupt mask
    fn interrupt_mask(&mut self, mask: u32);
    /// Retrieve currently runnign interrupts
    fn get_interrupts(&mut self) -> u32;
    /// Clear interrupt flags
    fn clear_interrupts(&mut self, mask: u32);
    /// Enable/disable loopback mode
    fn loopback(&mut self, state: bool);
    /// Enable/disable CAN-FD mode
    fn fd(&mut self, state: bool);
    /// Write a filter object to the standard filter block
    fn set_filter(&mut self, index: usize, filter: Filter) -> Result<()>;
    /// Write a filter object to the extended filter block
    fn set_ext_filter(&mut self, index: usize, filter: ExtFilter) -> Result<()>;
    /// Enable can device configuration mode
    fn enter_config_mode(&mut self);
    /// Enable can device operational mode
    fn enter_operational_mode(&mut self);
    /// Get current time
    fn ts_count(&self) -> u16;
}

/// A CAN bus
pub struct Can<'a, Id, D, C: Capacities> {
    /// CAN bus peripheral
    pub can: crate::reg::Can<Id>,
    config: RamConfig,
    /// For memory safety, all constructors must ensure that the memory is
    /// initialized.
    dependencies: D,
    memory: &'a mut SharedMemoryInner<C>,
    /// Controls enabling and line selection of interrupts.
    pub interrupts: InterruptConfiguration<Id>,
}

impl<Id: crate::CanId, D: crate::Dependencies<Id>, C: Capacities> Can<'_, Id, D, C> {
    /// Apply parameters from a bus config struct
    /// Safety: Config may only be applied safely if the bus is initializing,
    ///         if the bus is not initializing, an error is returned
    fn apply_bus_config(&mut self, config: &CanConfig, freq: HertzU32) -> Result<()> {
        if !(1..=16).contains(&config.timing.ts_prescale) {
            return Err(Error::InvalidTimeStampPrescaler);
        }

        if !self.can.cccr.read().init().bit() {
            return Err(Error::NotInitializing);
        } else {
            // Baud rate
            // TODO: rewrite this somewhat when we're required to implement variable data
            // rate!
            let c = self.dependencies.can_clock().to_Hz();
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
                    self.can.nbtp.write(|w| {
                        w.nsjw()
                            .bits(config.timing.sjw)
                            .ntseg1()
                            .bits(config.timing.phase_seg_1)
                            .ntseg2()
                            .bits(config.timing.phase_seg_2)
                            .nbrp()
                            .bits((divider - 1) as u16)
                    });

                    self.can.tscc.write(|w| {
                        w.tss()
                            .bits(config.timing.ts_select.into())
                            // Prescaler is 1 + tcp value.
                            .tcp()
                            .bits(config.timing.ts_prescale - 1)
                    });

                    // CAN-FD operation
                    self.can
                        .cccr
                        .modify(|_, w| w.fdoe().bit(config.fd_mode.clone().into()));
                    // HACK: Data bitrate is 1Mb/s
                    self.can.dbtp.modify(|_, w| w.dbrp().bits(2));
                    self.can
                        .cccr
                        .modify(|_, w| w.brse().bit(config.bit_rate_switching));
                    // Global filter options
                    self.can.gfc.write(|w| {
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
    fn apply_ram_config(&mut self) -> Result<()> {
        if !self.can.cccr.read().init().bit() {
            return Err(Error::NotInitializing);
        } else {
            let config = &self.config;
            let mem = &self.memory;

            unsafe {
                // Standard id
                self.can.sidfc.write(|w| {
                    w.flssa()
                        .bits(&mem.filters_standard as *const _ as u16)
                        .lss()
                        .bits(mem.filters_standard.len() as u8)
                });

                // Extended id
                self.can.xidfc.write(|w| {
                    w.flesa()
                        .bits(&mem.filters_extended as *const _ as u16)
                        .lse()
                        .bits(mem.filters_extended.len() as u8)
                });

                // RX buffers
                self.can
                    .rxbc
                    .write(|w| w.rbsa().bits(&mem.rx_dedicated_buffers as *const _ as u16));

                // Data field size for buffers and FIFOs
                self.can.rxesc.write(|w| {
                    w.rbds()
                        .bits(C::RxBufferMessage::REG)
                        .f0ds()
                        .bits(C::RxFifo0Message::REG)
                        .f1ds()
                        .bits(C::RxFifo1Message::REG)
                });

                //// RX FIFO 0
                self.can.rxf0.c.write(|w| {
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
                self.can.rxf1.c.write(|w| {
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
                self.can.txbc.write(|w| {
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
                self.can.txesc.write(|w| w.tbds().bits(C::TxMessage::REG));

                // TX events
                self.can.txefc.write(|w| {
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
        self.can.cccr.modify(|_, w| w.fdoe().bit(fd.clone().into()));
    }

    /// Configure test mode
    fn set_test(&mut self, test: TestMode) {
        match test {
            TestMode::Disabled => {
                self.can.cccr.modify(|_, w| w.test().bit(false));
                self.can.test.modify(|_, w| w.lbck().bit(false));
            }
            TestMode::Loopback => {
                self.can.cccr.modify(|_, w| w.test().bit(true));
                self.can.test.modify(|_, w| w.lbck().bit(true));
            }
        }
    }

    /// Signal add by writing a new request mask
    pub fn add_request(&mut self, mask: u32) -> Result<()> {
        unsafe {
            self.can.txbar.write(|w| w.bits(mask));
        }
        Ok(())

        // if (mask >> self.can.txbc.read().tfqs().bits()) == 0 {
        //     // TODO: this need s to be reworked!
        //     Err(Error::InvalidTxBuffer(mask))
        // } else {
        //     unsafe {
        //         self.can.txbar.write(|w| w.bits(mask));
        //     }
        //     Ok(())
        // }
    }

    /// Signal cancellation by writing a new request mask
    #[allow(unused)]
    fn cancel_request(&mut self, mask: u32) -> Result<()> {
        if (mask >> self.can.txbc.read().tfqs().bits()) == 0 {
            Err(Error::InvalidTxBuffer(mask))
        } else {
            unsafe {
                self.can.txbcr.write(|w| w.bits(mask));
            }
            Ok(())
        }
    }
}

impl<Id: crate::CanId, D: crate::Dependencies<Id>, C: Capacities> CanBus for Can<'_, Id, D, C> {
    fn error_counters(&self) -> ErrorCounters {
        self.can.ecr.read().into()
    }

    fn protocol_status(&self) -> ProtocolStatus {
        self.can.psr.read().into()
    }

    fn new_data(&self, index: usize) -> Result<bool> {
        if index < 32 {
            Ok(self.can.ndat1.read().bits() & (1 << index) > 0)
        } else if index < 64 {
            Ok(self.can.ndat1.read().bits() & (1 << (index - 32)) > 0)
        } else {
            Err(Error::InvalidBufferIndex)
        }
    }

    fn enable_interrupts(&mut self, mask: u32) {
        unsafe {
            self.can.ie.modify(|r, w| w.bits(r.bits() | mask));
        }
    }

    fn disable_interrupts(&mut self, mask: u32) {
        unsafe {
            self.can.ie.modify(|r, w| w.bits(r.bits() & !mask));
        }
    }

    fn interrupt_mask(&mut self, mask: u32) {
        unsafe {
            self.can.ie.write(|w| w.bits(mask));
        }
    }

    fn get_interrupts(&mut self) -> u32 {
        self.can.ir.read().bits()
    }

    fn clear_interrupts(&mut self, mask: u32) {
        unsafe {
            self.can.ir.write(|w| w.bits(mask));
        }
    }

    /// Set CAN FD mode
    fn fd(&mut self, state: bool) {
        self.enter_config_mode();

        // Enable configuration change
        self.can.cccr.modify(|_, w| w.cce().set_bit());
        while !self.can.cccr.read().cce().bit() {
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
        self.can.cccr.modify(|_, w| w.cce().set_bit());
        while !self.can.cccr.read().cce().bit() {
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

    fn set_filter(&mut self, index: usize, filter: Filter) -> Result<()> {
        self.memory
            .filters_standard
            .get_mut(index)
            .ok_or(Error::OutOfBounds)?
            .set(filter.into());
        Ok(())
    }

    fn set_ext_filter(&mut self, index: usize, filter: ExtFilter) -> Result<()> {
        self.memory
            .filters_extended
            .get_mut(index)
            .ok_or(Error::OutOfBounds)?
            .set(filter.into());
        Ok(())
    }

    /// Enter configuration mode
    fn enter_config_mode(&mut self) {
        self.can.cccr.modify(|_, w| w.init().set_bit());
        while !self.can.cccr.read().init().bit() {
            // TODO: Make sure this loop does not get optimized away
        }
    }

    /// Enter operational mode
    fn enter_operational_mode(&mut self) {
        // Finish initializing peripheral
        self.can.cccr.modify(|_, w| w.init().clear_bit());
        while self.can.cccr.read().init().bit() {
            // TODO: Make sure this loop does not get optimized away
        }
    }

    fn ts_count(&self) -> u16 {
        self.can.tscv.read().tsc().bits()
    }
}

impl<Id: crate::CanId, D: crate::Dependencies<Id>, C: Capacities> CanSendBuffer<C>
    for Can<'_, Id, D, C>
{
    fn transmit_buffer(&mut self, id: usize) -> Result<()> {
        self.add_request(1 << id)
    }

    fn send_buffer(&mut self, index: usize, message: C::TxMessage, auto_send: bool) -> Result<()> {
        if index > <C::DedicatedTxBuffers as Unsigned>::USIZE {
            return Err(Error::OutOfBounds);
        } else {
            self.memory
                .tx_buffers
                .get_mut(index)
                .ok_or(Error::OutOfBounds)?
                .set(message);

            if auto_send {
                // Send an add request for the current message
                Ok(self.transmit_buffer(index)?)
            } else {
                Ok(())
            }
        }
    }
}

impl<Id: crate::CanId, D: crate::Dependencies<Id>, C: Capacities> CanSendFifo<C>
    for Can<'_, Id, D, C>
{
    fn transmit_fifo(&mut self, index: usize) -> Result<()> {
        self.add_request(1 << index)
    }

    fn put(&mut self) -> Result<usize> {
        if self.can.txfqs.read().tfqf().bits() {
            return Err(Error::FifoFull);
        }
        Ok(self.can.txfqs.read().tfqpi().bits() as usize)
    }

    fn send_fifo(&mut self, message: C::TxMessage) -> Result<()> {
        // Get current put index
        let index = self.put()?;

        self.memory
            .tx_buffers
            .get_mut(index)
            .ok_or(Error::OutOfBounds)?
            .set(message);

        // Flag the add request
        self.transmit_fifo(index)
    }
}

impl<Id: crate::CanId, const N: usize, D: crate::Dependencies<Id>, C> CanSendSlice<C>
    for Can<'_, Id, D, C>
where
    C: Capacities<TxMessage = tx::Message<N>>,
{
    fn send_slice(&mut self, id: embedded_hal::can::Id, data: &[u8]) -> Result<()> {
        let message = tx::MessageBuilder {
            id,
            frame_contents: tx::FrameContents::Data(data),
            frame_format: tx::FrameFormat::Classic,
            store_tx_event: None,
        }
        .build()?;
        self.send_fifo(message)
    }
}

macro_rules! impl_fifo {
    ($fifo:ident, $message_type:ident, $mem_rx_fifo:ident, $rxf:ident) => {
        impl<Id: crate::CanId, D: crate::Dependencies<Id>, C: Capacities>
            CanReadFifo<$fifo, C, C::$message_type> for Can<'_, Id, D, C>
        {
            fn peek(&mut self) -> Result<C::$message_type> {
                if CanReadFifo::<$fifo, C, C::$message_type>::fill(self) <= 0 {
                    Err(Error::FifoEmpty)
                } else {
                    let index = CanReadFifo::<$fifo, C, C::$message_type>::get(self);
                    let m = self
                        .memory
                        .$mem_rx_fifo
                        .get(index)
                        .ok_or(Error::OutOfBounds)?
                        .get();
                    Ok(m)
                }
            }

            fn mark_fifo_read(&mut self) -> Result<()> {
                // Increment get index
                let get_index = self.can.$rxf.s.read().fgi().bits();
                unsafe {
                    self.can.$rxf.a.write(|w| w.fai().bits(get_index));
                }

                Ok(())
            }

            fn get(&self) -> usize {
                (self.can.$rxf.s.read().fgi().bits()) as usize
            }

            fn put(&self) -> usize {
                (self.can.$rxf.s.read().fpi().bits()) as usize
            }

            fn fill(&self) -> usize {
                (self.can.$rxf.s.read().ffl().bits()) as usize
            }

            fn free(&self) -> usize {
                // The maximum size according to the datasheet.
                const MAX_SIZE: u8 = 64;

                let reg_value = self.can.$rxf.c.read().fs().bits();
                let max_elems = usize::from(min(reg_value, MAX_SIZE));

                let fill = CanReadFifo::<$fifo, C, C::$message_type>::fill(self);

                max_elems.saturating_sub(fill)
            }
        }
    };
}

impl_fifo!(Fifo0, RxFifo0Message, rx_fifo_0, rxf0);
impl_fifo!(Fifo1, RxFifo1Message, rx_fifo_1, rxf1);

impl<Id: crate::CanId, D: crate::Dependencies<Id>, C: Capacities> CanReadBuffer<C>
    for Can<'_, Id, D, C>
{
    fn read_buffer(&mut self, index: usize) -> Result<C::RxBufferMessage> {
        if !self.new_data(index)? {
            return Err(Error::BufferDataNotNew);
        } else {
            let m = self
                .memory
                .rx_dedicated_buffers
                .get(index)
                .ok_or(Error::OutOfBounds)?
                .get();
            <Self as CanReadBuffer<C>>::mark_buffer_read(self, index)?;
            Ok(m)
        }
    }

    fn mark_buffer_read(&mut self, index: usize) -> Result<()> {
        match index {
            0..=31 => {
                if self.can.ndat1.read().bits() & (1 << index) != 0 {
                    unsafe {
                        self.can.ndat1.write(|w| w.bits(1 << index));
                    }
                } else {
                    return Err(Error::BufferDataNotNew);
                }
            }
            32..=63 => {
                if self.can.ndat2.read().bits() & (1 << (index >> 1)) != 0 {
                    unsafe {
                        self.can.ndat2.write(|w| w.bits(1 << (index >> 1)));
                    }
                } else {
                    return Err(Error::BufferDataNotNew);
                }
            }
            _ => return Err(Error::InvalidBufferIndex),
        }

        Ok(())
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
        let mut bus = Self {
            can,
            config: ram_cfg,
            dependencies,
            memory,
            // Safety: Since `new` takes a PAC singleton, it can only be called once. Then no
            // duplicate `InterruptConfiguration` will be constructed. The registers
            // that are delegated to `InterruptConfiguration` should not be touched by any other
            // code. This has to be upheld by all code that has access to the register block.
            interrupts: unsafe { InterruptConfiguration::new() },
        };

        bus.enter_config_mode();

        // Enable configuration change
        bus.can.cccr.modify(|_, w| w.cce().set_bit());
        while !bus.can.cccr.read().cce().bit() {
            // TODO: Make sure this loop does not get optimized away
        }

        // Apply RAM configuration
        bus.apply_ram_config().unwrap();

        // Apply additional CAN config
        bus.apply_bus_config(&can_cfg, freq).unwrap();

        bus.enter_operational_mode();

        Ok(bus)
    }
}
