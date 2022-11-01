//! Transmit buffer configuration

/// TX buffer const config
#[derive(Clone)]
pub struct Txbc {
    /// Action to take on overflow
    pub mode: TxBufferMode,
}

/// Event FIFO configuration
#[derive(Clone)]
pub struct Txefc {
    /// Fifo fullnes to generate interrupt
    pub watermark: u8,
}

/// How to treat the transmit buffer
#[derive(Clone)]
pub enum TxBufferMode {
    /// Act as a FIFO
    /// Messages are sent according to the get index
    Fifo,
    /// Act as a queue
    /// Messages are sent with priority according to lowest ID
    Queue,
}

impl From<TxBufferMode> for bool {
    fn from(val: TxBufferMode) -> Self {
        match val {
            TxBufferMode::Queue => true,
            TxBufferMode::Fifo => false,
        }
    }
}
