#![no_std]

pub mod reg;
pub use reg::CanId;

// For svd2rust generated code that refers to everything via `crate::...`
use reg::generic::*;
