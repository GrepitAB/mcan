//! Memory management for the RAM interface between core and peripheral.
use crate::filter::{FilterExtendedId, FilterStandardId};
use crate::message::{rx, tx, TxEvent};
use core::mem::MaybeUninit;
use generic_array::{
    typenum::{consts::*, IsLessOrEqual, LeEq, Same},
    ArrayLength, GenericArray,
};
use vcell::VolatileCell;

/// Start of addressable RAM
pub const SYSTEM_RAM: usize = 0x2000_0000;

/// Element capacities
pub trait Capacities {
    /// Maximum number of Standard ID filters
    type StandardFilters: LimitedArrayLength<VolatileCell<FilterStandardId>, U128>;
    /// Maximum number of Extended ID filters
    type ExtendedFilters: LimitedArrayLength<VolatileCell<FilterExtendedId>, U64>;
    /// [`rx::Message`] with size selected for use in dedicated receive buffers
    type RxBufferMessage: rx::AnyMessage;
    /// Maximum number of dedicated receive buffers
    type DedicatedRxBuffers: LimitedArrayLength<VolatileCell<Self::RxBufferMessage>, U64>;
    /// [`rx::Message`] with size selected for use in receive FIFO 0
    type RxFifo0Message: rx::AnyMessage;
    /// Receive FIFO0 size
    type RxFifo0: LimitedArrayLength<VolatileCell<Self::RxFifo0Message>, U64>;
    /// [`rx::Message`] with size selected for use in receive FIFO 1
    type RxFifo1Message: rx::AnyMessage;
    /// Receive FIFO1 size
    type RxFifo1: LimitedArrayLength<VolatileCell<Self::RxFifo1Message>, U64>;
    /// [`tx::Message`] with size selected for use in transmit buffers
    type TxMessage: tx::AnyMessage;
    /// Number of transmit buffers (later split into dedicated and queue use)
    type TxBuffers: LimitedArrayLength<VolatileCell<Self::TxMessage>, U32>;
    /// Number of transmit buffers to exempt from queue use to dedicate to
    /// specific messages. The rest are used as a queue.
    type DedicatedTxBuffers: LimitedArrayLength<VolatileCell<Self::TxMessage>, Self::TxBuffers>;
    /// Transmit event FIFO size
    type TxEventFifo: LimitedArrayLength<VolatileCell<TxEvent>, U32>;
}

/// [`generic_array::ArrayLength`] with an upper bound.
pub trait LimitedArrayLength<T, MaxLength>: ArrayLength<T> {}
impl<T, N, MaxLength> LimitedArrayLength<T, MaxLength> for N
where
    N: ArrayLength<T> + IsLessOrEqual<MaxLength>,
    LeEq<N, MaxLength>: Same<True>,
{
}

#[repr(C)]
pub(super) struct SharedMemoryInner<C: Capacities> {
    pub(super) filters_standard: GenericArray<VolatileCell<FilterStandardId>, C::StandardFilters>,
    pub(super) filters_extended: GenericArray<VolatileCell<FilterExtendedId>, C::ExtendedFilters>,
    pub(super) rx_fifo_0: GenericArray<VolatileCell<C::RxFifo0Message>, C::RxFifo0>,
    pub(super) rx_fifo_1: GenericArray<VolatileCell<C::RxFifo1Message>, C::RxFifo1>,
    pub(super) rx_dedicated_buffers:
        GenericArray<VolatileCell<C::RxBufferMessage>, C::DedicatedRxBuffers>,
    pub(super) tx_event_fifo: GenericArray<VolatileCell<TxEvent>, C::TxEventFifo>,
    pub(super) tx_buffers: GenericArray<VolatileCell<C::TxMessage>, C::TxBuffers>,
}

/// Memory shared between the peripheral and core. Provide a struct `C` that
/// implements [`Capacities`] to select the sizes of the buffers, then construct
/// this using `SharedMemory::<C>::new()`.
pub struct SharedMemory<C: Capacities>(MaybeUninit<SharedMemoryInner<C>>);

impl<C: Capacities> SharedMemory<C> {
    pub(super) fn init(&mut self) -> &mut SharedMemoryInner<C> {
        self.0 = MaybeUninit::zeroed();
        // Safety: All bits 0 is a valid value for all the contained arrays.
        unsafe { self.0.assume_init_mut() }
    }

    /// All initialization is handled by the type that uses the memory, so this
    /// type can safely be assigned to a link_section that is not
    /// initialized by the system to control its position in memory.
    pub const fn new() -> Self {
        Self(MaybeUninit::uninit())
    }

    /// The peripheral uses 16-bit addressing for its memory configuration,
    /// offset from the start of system RAM. If `SharedMemory` is allocated
    /// outside the addressable region, it cannot be used.
    pub fn is_addressable(&self) -> bool {
        let start = self as *const _ as usize;
        let end_exclusive = start + core::mem::size_of::<Self>();
        SYSTEM_RAM <= start && end_exclusive - SYSTEM_RAM <= 1 << 16
    }
}

impl<C: Capacities> SharedMemoryInner<C> {
    pub fn is_addressable(&self) -> bool {
        let start = self as *const _ as usize;
        let end_exclusive = start + core::mem::size_of::<Self>();
        SYSTEM_RAM <= start && end_exclusive - SYSTEM_RAM <= 1 << 16
    }
}
