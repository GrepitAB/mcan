//! Convenience re-export of common members
//!
//! Like the standard library's prelude, this module simplifies importing of
//! common items. Unlike the standard prelude, the contents of this module must
//! be imported manually:
//!
//! ```
//! use mcan::prelude::*;
//! ```

use crate::message::{self, rx, tx};
pub use message::Raw as _;
pub use rx::AnyMessage as _;
pub use tx::AnyMessage as _;

pub use crate::bus::DynAux as _;
pub use crate::rx_dedicated_buffers::DynRxDedicatedBuffer as _;
pub use crate::rx_fifo::DynRxFifo as _;
pub use crate::tx_buffers::DynTx as _;
pub use crate::tx_event_fifo::DynTxEventFifo as _;
