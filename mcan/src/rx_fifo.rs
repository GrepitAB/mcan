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

impl<'a, P: mcan_core::CanId, M: rx::AnyMessage> GetRxFifoRegs for RxFifo<'a, Fifo0, P, M> {
    unsafe fn registers(&self) -> &reg::RxFifoRegs {
        &(*P::register_block()).rxf0
    }
}
impl<'a, P: mcan_core::CanId, M: rx::AnyMessage> GetRxFifoRegs for RxFifo<'a, Fifo1, P, M> {
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

    /// Returns the number of elements in the queue
    pub fn len(&self) -> usize {
        self.regs().s.read().ffl().bits() as usize
    }

    /// Returns `true` if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of elements the queue can hold
    pub fn capacity(&self) -> usize {
        self.memory.len()
    }

    /// Returns a received frame if available. Note that the FIFO also
    /// implements [`Iterator`] to receive messages until the queue is empty.
    pub fn receive(&mut self) -> nb::Result<M, Infallible> {
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

impl<'a, F, P: mcan_core::CanId, M: rx::AnyMessage> Iterator for RxFifo<'a, F, P, M>
where
    Self: GetRxFifoRegs,
{
    type Item = M;

    fn next(&mut self) -> Option<Self::Item> {
        self.receive().ok()
    }
}
