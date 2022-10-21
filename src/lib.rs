#![no_std]

pub mod bus;
pub mod config;
pub mod filter;
pub mod message;
pub mod messageram;
pub mod reg;
pub mod rx_fifo;

pub use reg::CanId;

use fugit::HertzU32 as Hz;

// For svd2rust generated code that refers to everything via `crate::...`
use reg::generic::*;

// TODO: Documentation
/// # Safety
/// - Clocks must not change
/// - HW register referenced by `Id: CanId` has to be owned by struct
///   implementing this trait in order to avoid aliasing.
pub unsafe trait Dependencies<Id: CanId> {
    fn host_clock(&self) -> Hz;
    fn can_clock(&self) -> Hz;
}
