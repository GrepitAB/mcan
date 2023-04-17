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
