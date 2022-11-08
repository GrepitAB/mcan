#![no_std]
// TODO: Enable when documentation is finished
// #![warn(missing_docs)]

pub mod bus;
pub mod config;
pub mod filter;
pub mod interrupt;
pub mod message;
pub mod messageram;
pub mod reg;
pub mod rx_dedicated_buffers;
pub mod rx_fifo;
pub mod tx_buffers;
pub mod tx_event_fifo;

// For svd2rust generated code that refers to everything via `crate::...`
use reg::generic::*;
