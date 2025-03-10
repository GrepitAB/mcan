//! Queues for received messages
//!
//! Messages can be placed in the queues by filter [`Action`]s.
//!
//! [`Action`]: crate::filter::Action

use crate::message::rx;
use crate::reg;
use core::convert::Infallible;
use core::marker::PhantomData;
use reg::AccessRegisterBlock as _;
use vcell::VolatileCell;

/// Receive FIFO `F` on peripheral `P`.
pub struct RxFifo<'a, F, P, M: rx::AnyMessage> {
    memory: &'a mut [VolatileCell<M>],
    _markers: PhantomData<(F, P)>,
}

/// Trait which erases generic parametrization for [`RxFifo`] type
pub trait DynRxFifo {
    /// RX FIFO identity type
    type RxFifoId;

    /// CAN identity type
    type CanId;

    /// Received message type
    type Message;

    /// Returns the number of elements in the queue
    fn len(&self) -> usize;

    /// Returns `true` if the queue is empty
    fn is_empty(&self) -> bool;

    /// Returns the number of elements the queue can hold
    fn capacity(&self) -> usize;

    /// Returns a received frame if available. Note that the FIFO also
    /// implements [`Iterator`] to receive messages until the queue is empty.
    fn receive(&mut self) -> nb::Result<Self::Message, Infallible>;
}

/// Value of the type-level FIFO selection enum representing FIFO 0.
pub struct Fifo0;
/// Value of the type-level FIFO selection enum representing FIFO 1.
pub struct Fifo1;

/// Provides raw access to the registers controlling the RX FIFO.
pub trait GetRxFifoRegs {
    /// # Safety
    /// Direct access can break assumptions made by the abstraction.
    unsafe fn registers(&self) -> &reg::RxFifoRegs;
}

impl<P: mcan_core::CanId, M: rx::AnyMessage> GetRxFifoRegs for RxFifo<'_, Fifo0, P, M> {
    unsafe fn registers(&self) -> &reg::RxFifoRegs {
        &(*P::register_block()).rxf0
    }
}
impl<P: mcan_core::CanId, M: rx::AnyMessage> GetRxFifoRegs for RxFifo<'_, Fifo1, P, M> {
    unsafe fn registers(&self) -> &reg::RxFifoRegs {
        &(*P::register_block()).rxf1
    }
}

impl<'a, F, P: mcan_core::CanId, M: rx::AnyMessage> RxFifo<'a, F, P, M>
where
    Self: GetRxFifoRegs,
{
    /// # Safety
    /// The caller must be the owner or the peripheral referenced by `P`. The
    /// constructed type assumes ownership of some of the registers from the
    /// peripheral `RegisterBlock`. Do not use them to avoid aliasing. Do not
    /// keep multiple instances for the same FIFO and peripheral.
    /// - RXFC
    /// - RXFS
    /// - RXFA
    pub(crate) unsafe fn new(memory: &'a mut [VolatileCell<M>]) -> Self {
        Self {
            memory,
            _markers: PhantomData,
        }
    }

    fn regs(&self) -> &reg::RxFifoRegs {
        // Safety: The RxFifo owns the registers.
        unsafe { self.registers() }
    }
}

impl<F, P: mcan_core::CanId, M: rx::AnyMessage> DynRxFifo for RxFifo<'_, F, P, M>
where
    Self: GetRxFifoRegs,
{
    type RxFifoId = F;
    type CanId = P;
    type Message = M;

    fn len(&self) -> usize {
        self.regs().s.read().ffl().bits() as usize
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn capacity(&self) -> usize {
        self.memory.len()
    }

    fn receive(&mut self) -> nb::Result<Self::Message, Infallible> {
        let status = self.regs().s.read();
        let len = status.ffl().bits();
        if len == 0 {
            return Err(nb::Error::WouldBlock);
        }
        let get_index = status.fgi().bits() as usize;
        let message = self.memory[get_index].get();
        // Mark the message as read.
        // Safety: The written index must be valid since it was retrieved from the
        // peripheral, and the configuration was not changed.
        unsafe {
            self.regs().a.write(|w| w.fai().bits(get_index as u8));
        }
        Ok(message)
    }
}

impl<F, P: mcan_core::CanId, M: rx::AnyMessage> Iterator for RxFifo<'_, F, P, M>
where
    Self: GetRxFifoRegs,
{
    type Item = M;

    fn next(&mut self) -> Option<Self::Item> {
        self.receive().ok()
    }
}
