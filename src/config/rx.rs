//! Receive buffer configuration

/// Denotes a RX-fifo configuration
#[derive(Clone)]
pub struct Rxf {
    /// FIFO mode
    pub mode: RxFifoMode,
    /// Fifo fullnes to generate interrupt
    pub watermark: u8,
}

/// Operating modes for the two FIFO
#[derive(Clone)]
pub enum RxFifoMode {
    /// Blocking mode
    /// When the RX FIFO is full, not messages are written until at least one
    /// has been read out
    Blocking,
    /// Overwriting mode
    /// When the RX FIFO is full, the oldest messsage will be deleted and a new
    /// message will take its place
    Overwrite,
}

impl Into<bool> for RxFifoMode {
    fn into(self) -> bool {
        match self {
            RxFifoMode::Overwrite => true,
            RxFifoMode::Blocking => false,
        }
    }
}
