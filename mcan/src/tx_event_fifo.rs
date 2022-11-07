use crate::message::TxEvent;
use crate::reg;
use reg::AccessRegisterBlock as _;
use core::marker::PhantomData;
use vcell::VolatileCell;

/// Transmit event queue on peripheral `P`
pub struct TxEventFifo<'a, P> {
    memory: &'a mut [VolatileCell<TxEvent>],
    _markers: PhantomData<P>,
}

impl<'a, P: mcan_core::CanId> TxEventFifo<'a, P> {
    /// # Safety
    /// The caller must be the owner or the peripheral referenced by `P`. The
    /// constructed type assumes ownership of some of the registers from the
    /// peripheral `RegisterBlock`. Do not use them to avoid aliasing. Do not
    /// keep multiple instances for the same peripheral.
    /// - TXEFS
    /// - TXEFA
    pub(crate) unsafe fn new(memory: &'a mut [VolatileCell<TxEvent>]) -> Self {
        Self {
            memory,
            _markers: PhantomData,
        }
    }

    /// Raw access to the registers.
    unsafe fn regs(&self) -> &reg::RegisterBlock {
        &(*P::register_block())
    }

    fn txefs(&self) -> &reg::TXEFS {
        // Safety: `Self` owns the register.
        unsafe { &self.regs().txefs }
    }

    fn txefa(&self) -> &reg::TXEFA {
        // Safety: `Self` owns the register.
        unsafe { &self.regs().txefa }
    }

    /// Returns the number of elements in the queue
    pub fn len(&self) -> usize {
        self.txefs().read().effl().bits() as usize
    }

    /// Returns `true` if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of elements the queue can hold
    pub fn capacity(&self) -> usize {
        self.memory.len()
    }

    /// Takes the first event from the queue
    pub fn pop(&mut self) -> Option<TxEvent> {
        let status = self.txefs().read();
        if status.effl().bits() == 0 {
            None
        } else {
            let get_index = status.efgi().bits();
            let event = self.memory.get(get_index as usize)?.get();
            // Safety: The get index must be valid since it was retrieved from the
            // peripheral and the configuration has not changed.
            unsafe {
                self.txefa().write(|w| w.efai().bits(get_index));
            }
            Some(event)
        }
    }
}
