//! Information about successfully transmitted messages
//!
//! Events are only generated for messages with [`store_tx_event`] set.
//!
//! [`store_tx_event`]: crate::message::tx::MessageBuilder::store_tx_event
use crate::message::TxEvent;
use crate::reg;
use core::marker::PhantomData;
use reg::AccessRegisterBlock as _;
use vcell::VolatileCell;

/// Transmit event queue on peripheral `P`
pub struct TxEventFifo<'a, P> {
    memory: &'a mut [VolatileCell<TxEvent>],
    _markers: PhantomData<P>,
}

/// Trait which erases generic parametrization for [`TxEventFifo`] type
pub trait DynTxEventFifo {
    /// CAN identity type
    type Id;

    /// Returns the number of elements in the queue
    fn len(&self) -> usize;
    /// Returns `true` if the queue is empty
    fn is_empty(&self) -> bool;
    /// Returns the number of elements the queue can hold
    fn capacity(&self) -> usize;
    /// Takes the first event from the queue
    fn pop(&mut self) -> Option<TxEvent>;
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
}

impl<'a, P: mcan_core::CanId> DynTxEventFifo for TxEventFifo<'a, P> {
    type Id = P;

    fn len(&self) -> usize {
        self.txefs().read().effl().bits() as usize
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn capacity(&self) -> usize {
        self.memory.len()
    }

    fn pop(&mut self) -> Option<TxEvent> {
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
