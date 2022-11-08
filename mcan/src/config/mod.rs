//! Configuration fields

pub mod bus;
pub mod rx;
pub mod tx;

use rx::Rxf;
use tx::{Txbc, Txefc};

/// Static memory map for the CAN bus configuration
#[derive(Clone)]
pub struct RamConfig {
    /// RX Fifo 0
    pub rxf0: Rxf,
    /// RX Fifo 1
    pub rxf1: Rxf,
    /// TX Event
    pub txefc: Txefc,
    /// TX Buffer
    pub txbc: Txbc,
}
