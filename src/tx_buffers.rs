use crate::reg;
use crate::{bus, messageram::Capacities};
use core::marker::PhantomData;
use generic_array::{typenum::Unsigned, GenericArray};
use vcell::VolatileCell;

/// Transmit queue and dedicated buffers
pub struct Tx<'a, P, C: Capacities> {
    memory: &'a mut GenericArray<VolatileCell<C::TxMessage>, C::TxBuffers>,
    _markers: PhantomData<P>,
}

impl<'a, P: crate::CanId, C: Capacities> Tx<'a, P, C> {
    /// # Safety
    /// The caller must be the owner or the peripheral referenced by `P`. The
    /// constructed type assumes ownership of some of the registers from the
    /// peripheral `RegisterBlock`. Do not use them to avoid aliasing. Do not
    /// keep multiple instances for the same peripheral.
    /// - TXFQS
    /// - TXBRP
    /// - TXBAR
    /// - TXBCR
    /// - TXBTO
    /// - TXBCF
    /// - TXBTIE
    /// - TXBCIE
    pub(crate) unsafe fn new(
        memory: &'a mut GenericArray<VolatileCell<C::TxMessage>, C::TxBuffers>,
    ) -> Self {
        Self {
            memory,
            _markers: PhantomData,
        }
    }

    /// Raw access to the registers.
    unsafe fn regs(&self) -> &reg::RegisterBlock {
        &(*P::ADDRESS)
    }

    fn txfqs(&self) -> &reg::TXFQS {
        // Safety: `Self` owns the register.
        unsafe { &self.regs().txfqs }
    }

    fn txbrp(&self) -> &reg::TXBRP {
        // Safety: `Self` owns the register.
        unsafe { &self.regs().txbrp }
    }

    fn txbar(&self) -> &reg::TXBAR {
        // Safety: `Self` owns the register.
        unsafe { &self.regs().txbar }
    }

    fn txbcr(&self) -> &reg::TXBCR {
        // Safety: `Self` owns the register.
        unsafe { &self.regs().txbcr }
    }

    fn txbto(&self) -> &reg::TXBTO {
        // Safety: `Self` owns the register.
        unsafe { &self.regs().txbto }
    }

    fn txbcf(&self) -> &reg::TXBCF {
        // Safety: `Self` owns the register.
        unsafe { &self.regs().txbcf }
    }

    fn txbtie(&self) -> &reg::TXBTIE {
        // Safety: `Self` owns the register.
        unsafe { &self.regs().txbtie }
    }

    fn txbcie(&self) -> &reg::TXBCIE {
        // Safety: `Self` owns the register.
        unsafe { &self.regs().txbcie }
    }

    fn add_request(&self, index: usize) {
        // Safety: There are no reserved bit patterns. According to the datasheet,
        // "TXBAR bits are set only for those Tx Buffers configured via TXBC". Our
        // interpretation is that add requests for buffers not configured in TXBC are
        // ignored.
        unsafe { self.txbar().write(|w| w.bits(1 << index)) }
    }

    fn is_buffer_in_use(&self, index: usize) -> bool {
        // It is unclear from the datasheet when BRP is updated. It is hopefully done
        // before clearing BAR, so that we don't get any false "not in use" from this.
        let add_requests = self.txbar().read().bits();
        let pending = self.txbrp().read().bits();
        (add_requests | pending) & (1 << index) != 0
    }

    /// Puts a frame in the specified transmit buffer to be sent on the bus.
    /// Fails with [`nb::Error::WouldBlock`] if the transmit buffer is full.
    fn transmit(
        &mut self,
        index: usize,
        message: C::TxMessage,
    ) -> nb::Result<(), bus::OutOfBounds> {
        if self.is_buffer_in_use(index) {
            return Err(nb::Error::WouldBlock);
        }
        self.memory
            .get_mut(index)
            .ok_or(bus::OutOfBounds)?
            .set(message);
        self.add_request(index);
        Ok(())
    }

    /// Puts a frame in the specified dedicated transmit buffer to be sent on
    /// the bus. Fails with [`nb::Error::WouldBlock`] if the transmit buffer
    /// is full.
    pub fn transmit_dedicated(
        &mut self,
        index: usize,
        message: C::TxMessage,
    ) -> nb::Result<(), bus::OutOfBounds> {
        if index > C::DedicatedTxBuffers::USIZE {
            Err(bus::OutOfBounds)?;
        }
        self.transmit(index, message)
    }

    /// Returns the put index if available. `None` if the queue is full.
    fn find_put_index(&self) -> Option<usize> {
        let status = self.txfqs().read();
        if status.tfqf().bit() {
            None
        } else {
            Some(status.tfqpi().bits() as usize)
        }
    }

    /// Puts a frame in the queue to be sent on the bus.
    /// Fails with [`nb::Error::WouldBlock`] if the transmit buffer is full.
    pub fn transmit_queued(&mut self, message: C::TxMessage) -> nb::Result<(), bus::OutOfBounds> {
        let index = self.find_put_index().ok_or(nb::Error::WouldBlock)?;
        self.transmit(index, message)
    }

    /// Allow [`Interrupt::TransmissionCancellationFinished`] to be triggered by
    /// `buffers`. Interrupts for other buffers remain unchanged.
    ///
    /// Note that the peripheral-level interrupt also needs to be enabled for
    /// interrupts to reach the system interrupt controller.
    ///
    /// [`Interrupt::TransmissionCancellationFinished`]: crate::config::bus::Interrupt::TransmissionCancellationFinished
    pub fn enable_cancellation_interrupt(&mut self, buffers: TxBufferSet) {
        // Safety: There are no reserved bit patterns.
        unsafe {
            self.txbcie().modify(|r, w| w.bits(r.bits() | buffers.0));
        }
    }

    /// Disallow [`Interrupt::TransmissionCancellationFinished`] to be triggered
    /// by `buffers`. Interrupts for other buffers remain unchanged.
    ///
    /// [`Interrupt::TransmissionCancellationFinished`]: crate::config::bus::Interrupt::TransmissionCancellationFinished
    pub fn disable_cancellation_interrupt(&mut self, buffers: TxBufferSet) {
        // Safety: There are no reserved bit patterns.
        unsafe {
            self.txbcie().modify(|r, w| w.bits(r.bits() & !buffers.0));
        }
    }

    /// Allow [`Interrupt::TransmissionCompleted`] to be triggered by `buffers`.
    /// Interrupts for other buffers remain unchanged.
    ///
    /// Note that the peripheral-level interrupt also needs to be enabled for
    /// interrupts to reach the system interrupt controller.
    ///
    /// [`Interrupt::TransmissionCompleted`]: crate::config::bus::Interrupt::TransmissionCompleted
    pub fn enable_transmission_completed_interrupt(&mut self, buffers: TxBufferSet) {
        // Safety: There are no reserved bit patterns.
        unsafe {
            self.txbtie().modify(|r, w| w.bits(r.bits() | buffers.0));
        }
    }

    /// Disallow [`Interrupt::TransmissionCompleted`] to be triggered by
    /// `buffers`. Interrupts for other buffers remain unchanged.
    ///
    /// [`Interrupt::TransmissionCompleted`]: crate::config::bus::Interrupt::TransmissionCompleted
    pub fn disable_transmission_completed_interrupt(&mut self, buffers: TxBufferSet) {
        // Safety: There are no reserved bit patterns.
        unsafe {
            self.txbtie().modify(|r, w| w.bits(r.bits() & !buffers.0));
        }
    }

    /// Returns the set of `TxBuffer`s that the peripheral indicates have been
    /// cancelled. The flags are only cleared when a new transmission is
    /// requested for the buffer.
    pub fn get_cancellation_flags(&self) -> TxBufferSet {
        TxBufferSet(self.txbcf().read().bits())
    }

    /// Returns the set of `TxBuffer`s that the peripheral indicates have been
    /// successfully transmitted. The flags are only cleared when a new
    /// transmission is requested for the buffer.
    pub fn get_transmission_completed_flags(&self) -> TxBufferSet {
        TxBufferSet(self.txbto().read().bits())
    }

    /// Returns an iterator over the set of `TxBuffer`s that the peripheral
    /// indicates have been cancelled. The flags are only cleared when a new
    /// transmission is requested for the buffer.
    pub fn iter_cancellation_flags(&self) -> Iter {
        self.get_cancellation_flags().iter()
    }

    /// Returns an iterator over the set of `TxBuffer`s that the peripheral
    /// indicates have been successfully transmitted. The flags are only cleared
    /// when a new transmission is requested for the buffer.
    pub fn iter_transmission_completed_flags(&self) -> Iter {
        self.get_transmission_completed_flags().iter()
    }

    /// Request cancellation of `buffers`. This function does not wait for
    /// cancellation to finish. Successful cancellation is indicated by a
    /// combination of the transmission completed and cancellation flags. If
    /// the flags indicate that transmission is completed, then the request
    /// arrived too late to stop the transmission, which was completed
    /// successfully. If the cancellation flag is set, but not the
    /// transmission completed flag, the transmission was either not started or
    /// was aborted due to an error.
    pub fn cancel_multi(&mut self, buffers: TxBufferSet) {
        // Safety: There are no reserved bit patterns.
        unsafe {
            self.txbcr().write(|w| w.bits(buffers.0));
        }
    }

    /// Request cancellation of a transmit buffer. This function does not wait
    /// for cancellation to finish. See [`Self::cancel_multi`].
    pub fn cancel(&mut self, index: usize) {
        self.cancel_multi([index].into_iter().collect())
    }
}

/// A set of transmit buffers, which may be dedicated buffers or part of the
/// queue.
#[derive(Copy, Clone)]
pub struct TxBufferSet(pub u32);
impl FromIterator<usize> for TxBufferSet {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        let mut set = 0_u32;
        for i in iter.into_iter() {
            set |= 1u32 << i;
        }
        TxBufferSet(set)
    }
}

impl TxBufferSet {
    /// Returns the set of all transmit buffers
    pub fn all() -> Self {
        Self(u32::MAX)
    }

    /// An iterator visiting all elements in arbitrary order.
    pub fn iter(&self) -> Iter {
        Iter {
            flags: *self,
            index: 0,
        }
    }
}

/// An iterator over the buffer indexes of the buffers in a [`TxBufferSet`].
///
/// This `struct` is created by [`TxBufferSet::iter`].
pub struct Iter {
    flags: TxBufferSet,
    index: u8,
}

impl Iterator for Iter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.index;
        self.index = self.index.saturating_add(1);
        if i > 31 {
            None
        } else if self.flags.0 & (1 << i) != 0 {
            Some(i as usize)
        } else {
            self.next()
        }
    }
}
