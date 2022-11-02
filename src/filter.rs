//! Message filters
use core::marker::PhantomData;
use embedded_can::{ExtendedId, StandardId};
use vcell::VolatileCell;

pub type FiltersStandard<'a, P> = Filters<'a, P, FilterStandardId>;
pub type FiltersExtended<'a, P> = Filters<'a, P, FilterExtendedId>;

/// Acceptance filters for incoming messages
pub struct Filters<'a, P, T> {
    memory: &'a mut [VolatileCell<T>],
    len: usize,
    _markers: PhantomData<P>,
}

impl<'a, P, T: Copy> Filters<'a, P, T> {
    /// # Safety
    /// All filters are assumed to be disabled initially. This is the case if
    /// the memory is zeroed.
    ///
    /// Notably, `Filters` does not assume ownership over the filter-related
    /// registers, as we need to know we are in initialization mode for their
    /// access to be safe.
    pub(crate) unsafe fn new(memory: &'a mut [VolatileCell<T>]) -> Self {
        Self {
            memory,
            len: 0,
            _markers: PhantomData,
        }
    }

    /// Overwrites the `filter` at `index`.
    /// Returns back the `filter` if the `index` is out of range.
    fn set<F: Copy + Into<T>>(&mut self, index: usize, filter: F) -> Result<(), F> {
        self.memory
            .get_mut(index)
            .map(|f| f.set(filter.into()))
            .ok_or(filter)
    }
    /// Appends a `filter` to the back of the list. Returns the assigned index
    /// if successful. Returns back the `filter` if the list is full.
    pub fn push<F: Copy + Into<T>>(&mut self, filter: F) -> Result<usize, F> {
        let index = self.len;
        self.set(index, filter)?;
        self.len += 1;
        Ok(index)
    }
}

/// 11-bit filter in the peripheral's representation
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FilterStandardId(pub(super) u32);
/// 29-bit filter in the peripheral's representation
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FilterExtendedId(pub(super) [u32; 2]);

/// Message filter field for 11-bit RX messages
#[derive(Copy, Clone)]
pub enum Filter {
    /// The filter is skipped
    Disabled,
    /// Range filter from low to high IDs
    Range {
        /// Action to take on a matched element
        action: Action,
        /// Lower filter limit
        low: StandardId,
        /// Upper filter limit
        high: StandardId,
    },
    /// Filter for two IDs
    Dual {
        /// Action to take on a matched element
        action: Action,
        /// Individual filter 1
        id1: StandardId,
        /// Individual filter 2
        id2: StandardId,
    },
    /// Traditional filter/mask CAN filter
    Classic {
        /// Action to take on a matched element
        action: Action,
        /// ID filter
        filter: StandardId,
        /// ID mask
        mask: StandardId,
    },
    /// Store into RX buffer or as debug message (ignores filter type)
    /// NOTE: Filter event pins are currently unsupported
    StoreBuffer {
        /// 11-bit filter ID 1
        id: StandardId,
        /// Special message type for StoreRxBuffer
        msg_type: SbMsgType,
        /// Offset to Rx buffer SA for
        offset: u8,
    },
}

/// Store buffer message types
#[derive(Copy, Clone)]
pub enum SbMsgType {
    /// Store into RX buffer slot poitner to by id
    RxBuffer = 0,
    /// Debug message A
    DebugA,
    /// Debug message D
    DebugB,
    /// Debug message C
    DebugC,
}

impl Default for SbMsgType {
    fn default() -> Self {
        Self::RxBuffer
    }
}

/// Message filter field for 28-bit RX messages
#[derive(Copy, Clone)]
pub enum ExtFilter {
    /// The filter is skipped
    Disabled,
    /// Range filter from low to high IDs with XIDAM
    MaskedRange {
        /// Action to take on a matched element
        action: Action,
        /// Lower filter limit
        low: ExtendedId,
        /// Upper filter limit
        high: ExtendedId,
    },
    /// Filter for two IDs
    Dual {
        /// Action to take on a matched element
        action: Action,
        /// Individual filter 1
        id1: ExtendedId,
        /// Individual filter 2
        id2: ExtendedId,
    },
    /// Traditional filter/mask CAN filter
    Classic {
        /// Action to take on a matched element
        action: Action,
        /// ID filter
        filter: ExtendedId,
        /// ID mask
        mask: ExtendedId,
    },
    /// Range filter from low to high IDs without XIDAM
    Range {
        /// Action to take on a matched element
        action: Action,
        /// Lower filter limit
        low: ExtendedId,
        /// Upper filter limit
        high: ExtendedId,
    },
    /// Store into RX buffer or as debug message (ignores filter type)
    /// NOTE: Filter event pins are currently unsupported
    StoreBuffer {
        /// 29-bit filter ID 1
        id: ExtendedId,
        /// Special message type for StoreRxBuffer
        msg_type: SbMsgType,
        /// Offset to Rx buffer SA for
        offset: u8,
    },
}

/// Filter element configurations
#[derive(Copy, Clone)]
pub enum Action {
    /// Store in RX FIFO 0 if filter matches
    StoreFifo0,
    /// Store in RX FIFO 1 if filter matches
    StoreFifo1,
    /// Reject ID if filter matches
    Reject,
    /// Set priority if filter matches
    Priority,
    /// Set priority and store in FIFO 0 if filter matches
    PriorityFifo0,
    /// Set priority and store in FIFO 1 if filter matches
    PriorityFifo1,
}

impl From<Action> for u32 {
    fn from(val: Action) -> Self {
        match val {
            Action::StoreFifo0 => 0x1,
            Action::StoreFifo1 => 0x2,
            Action::Reject => 0x3,
            Action::Priority => 0x4,
            Action::PriorityFifo0 => 0x5,
            Action::PriorityFifo1 => 0x6,
        }
    }
}

impl From<Filter> for FilterStandardId {
    fn from(val: Filter) -> Self {
        let v = match val {
            Filter::Disabled => 0,
            Filter::Range { action, high, low } => {
                let action: u32 = action.into();

                (high.as_raw() as u32) | ((low.as_raw() as u32) << 16) | (action << 27)
            }
            Filter::Dual { action, id1, id2 } => {
                let action: u32 = action.into();

                (id2.as_raw() as u32) | ((id1.as_raw() as u32) << 16) | (action << 27) | (1 << 30)
            }
            Filter::Classic {
                action,
                filter,
                mask,
            } => {
                let action: u32 = action.into();

                (mask.as_raw() as u32)
                    | ((filter.as_raw() as u32) << 16)
                    | (action << 27)
                    | (2 << 30)
            }
            Filter::StoreBuffer {
                id,
                msg_type,
                offset,
            } => (id.as_raw() as u32) << 16 | (msg_type as u32) << 9 | offset as u32 | (0x7 << 27),
        };

        FilterStandardId(v)
    }
}

impl From<ExtFilter> for FilterExtendedId {
    fn from(val: ExtFilter) -> Self {
        let (v1, v2) = match val {
            ExtFilter::Disabled => (0, 0),
            ExtFilter::MaskedRange { action, high, low } => {
                let action: u32 = action.into();

                ((action << 29 | low.as_raw()), high.as_raw())
            }
            ExtFilter::Dual { action, id1, id2 } => {
                let action: u32 = action.into();

                ((action << 29 | id1.as_raw()), (1 << 30 | id2.as_raw()))
            }
            ExtFilter::Classic {
                action,
                filter,
                mask,
            } => {
                let action: u32 = action.into();

                ((action << 29 | filter.as_raw()), (2 << 30 | mask.as_raw()))
            }
            ExtFilter::Range { action, high, low } => {
                let action: u32 = action.into();

                ((action << 29 | low.as_raw()), (3 << 30 | high.as_raw()))
            }
            ExtFilter::StoreBuffer {
                id,
                msg_type,
                offset,
            } => (
                (0x7 << 29 | id.as_raw()),
                (msg_type as u32) << 9 | offset as u32,
            ),
        };
        FilterExtendedId([v1, v2])
    }
}
