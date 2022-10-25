//! Message filters
use super::messageram;
use embedded_can::{ExtendedId, StandardId};

/// Message filter field for 11-bit RX messages
pub enum Filter {
    /// Range filter from low to high IDs
    Range {
        /// Action to take on a matched element
        action: ElementConfig,
        /// Lower filter limit
        low: StandardId,
        /// Upper filter limit
        high: StandardId,
    },
    /// Filter for two IDs
    Dual {
        /// Action to take on a matched element
        action: ElementConfig,
        /// Individual filter 1
        id1: StandardId,
        /// Individual filter 2
        id2: StandardId,
    },
    /// Traditional filter/mask CAN filter
    Classic {
        /// Action to take on a matched element
        action: ElementConfig,
        /// ID filter
        filter: StandardId,
        /// ID mask
        mask: StandardId,
    },
    /// Store into RX buffer or as debug message (ignores filter type)
    /// NOTE: Filter event pins SFID 8:6  are ignored for now
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
pub enum ExtFilter {
    /// Range filter from low to high IDs with XIDAM
    MaskedRange {
        /// Action to take on a matched element
        action: ElementConfig,
        /// Lower filter limit
        low: ExtendedId,
        /// Upper filter limit
        high: ExtendedId,
    },
    /// Filter for two IDs
    Dual {
        /// Action to take on a matched element
        action: ElementConfig,
        /// Individual filter 1
        id1: ExtendedId,
        /// Individual filter 2
        id2: ExtendedId,
    },
    /// Traditional filter/mask CAN filter
    Classic {
        /// Action to take on a matched element
        action: ElementConfig,
        /// ID filter
        filter: ExtendedId,
        /// ID mask
        mask: ExtendedId,
    },
    /// Range filter from low to high IDs without XIDAM
    Range {
        /// Action to take on a matched element
        action: ElementConfig,
        /// Lower filter limit
        low: ExtendedId,
        /// Upper filter limit
        high: ExtendedId,
    },
    /// Store into RX buffer or as debug message (ignores filter type)
    /// NOTE: Filter event pins SFID 8:6  are ignored for now
    StoreBuffer {
        /// 11-bit filter ID 1
        id: ExtendedId,
        /// Special message type for StoreRxBuffer
        msg_type: SbMsgType,
        /// Offset to Rx buffer SA for
        offset: u8,
    },
}

/// Filter element configurations
pub enum ElementConfig {
    /// Disable filter element
    Disable,
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

impl Into<u32> for ElementConfig {
    fn into(self) -> u32 {
        match self {
            ElementConfig::Disable => 0x0,
            ElementConfig::StoreFifo0 => 0x1,
            ElementConfig::StoreFifo1 => 0x2,
            ElementConfig::Reject => 0x3,
            ElementConfig::Priority => 0x4,
            ElementConfig::PriorityFifo0 => 0x5,
            ElementConfig::PriorityFifo1 => 0x6,
        }
    }
}

impl Into<messageram::FilterStandardId> for Filter {
    fn into(self) -> messageram::FilterStandardId {
        let v = match self {
            Filter::Range { action, high, low } => {
                let action: u32 = action.into();

                (high.as_raw() as u32) | ((low.as_raw() as u32) << 16) | (action << 27) | (0 << 30)
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
            } => {
                (id.as_raw() as u32) << 16
                    | (msg_type as u32) << 9
                    | (offset << 0) as u32
                    | (0x7 << 27)
                    | (0 << 30) // NOTE: ignored since FEC=STRXBUF
            }
        };

        messageram::FilterStandardId(v)
    }
}

impl Into<messageram::FilterExtendedId> for ExtFilter {
    fn into(self) -> messageram::FilterExtendedId {
        let (v1, v2) = match self {
            ExtFilter::MaskedRange { action, high, low } => {
                let action: u32 = action.into();

                ((action << 29 | low.as_raw()), (0 << 30 | high.as_raw()))
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
                (msg_type as u32) << 9 | (offset << 0) as u32,
            ),
        };
        messageram::FilterExtendedId([v1, v2])
    }
}
