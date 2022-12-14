#![no_std]
#![warn(missing_docs)]
//! # MCAN
//!
//! ## Overview
//! This crate provides a platform-agnostic CAN HAL.
//!
//! It provides the following features:
//!
//! - classical CAN and CAN FD with bitrate switching support
//! - safe Message RAM layouting system via [`SharedMemory`] that prevents
//!   misconfiguration through compile-time checks
//! - modular interrupt handling abstractions that enable lock-less usage of HW
//!   interrupt lines
//! - message transmission using dedicated buffers, FIFO and priority queue
//! - message transmission cancellation
//! - message reception using dedicated buffers and two FIFOs
//! - filter settings
//!
//! MCAN is embedded in the MCU like all other peripherals. The interface
//! between them includes two clock signal lines, two HW interrupt lines, a
//! single memory-mapped HW register and a dedicated, shared RAM memory region
//! (referred to as Message RAM) that both CPU and MCAN can access and share
//! information through.
//!
//! For the MCAN abstractions to be considered operational, this interface has
//! to be properly configured. The latter is assured through the safety
//! requirements of [`mcan_core`] traits which platform-specific HALs are
//! expected to implement.
//!
//! As a result, application developer, in order to instantiate
//! [`CanConfigurable`] an instance of a [`mcan_core::Dependencies`]
//! implementing struct is required and according to the safety requirements of
//! the latter, MCAN prerequisites should be upheld for as long as it exists.
//!
//! ## Message RAM Configuration
//!
//! All platform specific details are covered by
//! [`mcan_core::Dependencies`] **apart from** the Message RAM configuration.
//! This is because it is hard to enclose all safety requirements for shared
//! message RAM on a platform-specific HAL level.
//!
//! MCAN uses 16-bit addressing internally. It means that all higher bytes of
//! the addressing has to be configured externally to MCAN HAL.
//! [`mcan_core::Dependencies::eligible_message_ram_start`] implemented by
//! platform-specific HAL provides a way to `mcan` to verify if the memory
//! region provided by a user is sound; yet it is up to the user to put it in a
//! valid, accessible to MCAN, RAM memory region.
//!
//! Standard way of configuring Message RAM would be to
//! - specify custom MEMORY entry in a linker script mapped to valid RAM memory
//!   region
//! - introduce custom, `.bss` like (NOLOAD property), section - eg. `.can`
//! - map new custom input section to new custom MEMORY entry
//! - use `#[link_section]` attribute in a code to link static variable to this
//!   memory region
//!
//! Example of a linker script
//! ```text
//! MEMORY
//! {
//!   FLASH : ORIGIN = 0x400000, LENGTH = 2M
//!   CAN : ORIGIN = 0x20400000, LENGTH = 64K
//!   RAM : ORIGIN = 0x20410000, LENGTH = 192K
//! }
//!
//! SECTIONS {
//!   .can (NOLOAD) :
//!   {
//!     *(.can .can.*);
//!   } > CAN
//! }
//! ```
//!
//! Code example
//!
//! ```no_run
//! use mcan::generic_array::typenum::consts::*;
//! use mcan::messageram::SharedMemory;
//! use mcan::message::{tx, rx};
//! use mcan::prelude::*;
//! struct Capacities;
//! impl mcan::messageram::Capacities for Capacities {
//!     type StandardFilters = U128;
//!     type ExtendedFilters = U64;
//!     type RxBufferMessage = rx::Message<64>;
//!     type DedicatedRxBuffers = U64;
//!     type RxFifo0Message = rx::Message<64>;
//!     type RxFifo0 = U64;
//!     type RxFifo1Message = rx::Message<64>;
//!     type RxFifo1 = U64;
//!     type TxMessage = tx::Message<64>;
//!     type TxBuffers = U32;
//!     type DedicatedTxBuffers = U0;
//!     type TxEventFifo = U32;
//! }
//!
//! #[link_section = ".can"]
//! static mut MESSAGE_RAM: SharedMemory<Capacities> = SharedMemory::new();
//! ```
//!
//! When it comes to [`RTIC`] framework, suggested way of setting the shared
//! memory up would be to use task-local resource in an `init` task. Reference
//! to a task-local resource in an `init` has a static lifetime which is
//! suitable for configuring MCAN returning it from `init`.
//!
//! ```ignore
//! #[rtic::app(device = hal::pac, peripherals = true, dispatchers = [SOME_DISPATCHER])]
//! mod app {
//!    #[init(local = [
//!        #[link_section = ".can"]
//!        message_ram: SharedMemory<Capacities> = SharedMemory::new()
//!    ])]
//!    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
//!        // `ctx.local.message_ram` is a reference with a static lifetime
//!        // ...
//!    }
//! }
//! ```
//!
//! ## General usage example
//!
//! In order to use MCAN abstractions one has to
//! - instantiate [`mcan_core::Dependencies`] implementing struct
//! - setup message RAM
//!
//! ```no_run
//! # use mcan::generic_array::typenum::consts::*;
//! # use mcan::messageram::SharedMemory;
//! # use mcan::message::{tx, rx};
//! # use mcan::prelude::*;
//! # use fugit::RateExtU32 as _;
//! # struct Capacities;
//! # impl mcan::messageram::Capacities for Capacities {
//! #     type StandardFilters = U128;
//! #     type ExtendedFilters = U64;
//! #     type RxBufferMessage = rx::Message<64>;
//! #     type DedicatedRxBuffers = U64;
//! #     type RxFifo0Message = rx::Message<64>;
//! #     type RxFifo0 = U64;
//! #     type RxFifo1Message = rx::Message<64>;
//! #     type RxFifo1 = U64;
//! #     type TxMessage = tx::Message<64>;
//! #     type TxBuffers = U32;
//! #     type DedicatedTxBuffers = U0;
//! #     type TxEventFifo = U32;
//! # }
//! # #[link_section = ".can"]
//! # static mut MESSAGE_RAM: SharedMemory<Capacities> = SharedMemory::new();
//! # struct Can0;
//! # unsafe impl mcan::core::CanId for Can0 {
//! #     const ADDRESS: *const () = 0xDEAD0000 as *const _;
//! # }
//! # pub mod hal {
//! #     pub mod can {
//! #         pub struct Dependencies(());
//! #         unsafe impl<ID: mcan::core::CanId> mcan::core::Dependencies<ID> for Dependencies {
//! #             fn eligible_message_ram_start(&self) -> *const () { unreachable!() }
//! #             fn host_clock(&self) -> fugit::HertzU32 { unreachable!() }
//! #             fn can_clock(&self) -> fugit::HertzU32 { unreachable!() }
//! #         }
//! #         impl Dependencies {
//! #             pub fn new() -> Result<Dependencies, ()> {
//! #                 Ok(Dependencies(()))
//! #             }
//! #         }
//! #     }
//! # }
//! use mcan::config::{BitTiming, Mode};
//! use mcan::interrupt::{Interrupt, InterruptLine};
//! use mcan::filter::{Action, Filter, ExtFilter};
//! use mcan::embedded_can as ecan;
//!
//! let dependencies = hal::can::Dependencies::new(/* all required parameters */).unwrap();
//! let mut can = mcan::bus::CanConfigurable::<'_, Can0, _, _>::new(
//!     500.kHz(),
//!     dependencies,
//!     unsafe { &mut MESSAGE_RAM }
//! ).unwrap();
//!
//! // MCAN is still disabled and user can access and modify the underlying
//! // config struct. More information can be found in `mcan::config` module.
//! can.config().mode = Mode::Fd {
//!     allow_bit_rate_switching: true,
//!     data_phase_timing: BitTiming::new(1.MHz()),
//! };
//!
//! // Example interrupt configuration
//! let line_0_interrupts = can
//!     .interrupts()
//!     .enable(
//!         [
//!             Interrupt::RxFifo0NewMessage,
//!             Interrupt::RxFifo0Full,
//!             Interrupt::RxFifo0MessageLost,
//!         ]
//!         .into_iter()
//!         .collect(),
//!         InterruptLine::Line0,
//!     )
//!     .unwrap();
//!
//! let line_1_interrupts = can
//!     .interrupts()
//!     .enable(
//!         [
//!             Interrupt::RxFifo1NewMessage,
//!             Interrupt::RxFifo1Full,
//!             Interrupt::RxFifo1MessageLost,
//!         ]
//!         .into_iter()
//!         .collect(),
//!         InterruptLine::Line1,
//!     )
//!     .unwrap();
//!
//! // Example filters configuration
//! // This filter will put all messages with a standard ID into RxFifo0
//! can.filters_standard()
//!     .push(Filter::Classic {
//!         action: Action::StoreFifo0,
//!         filter: ecan::StandardId::MAX,
//!         mask: ecan::StandardId::ZERO,
//!     })
//!     .unwrap_or_else(|_| panic!("Standard filter application failed"));
//!
//! // This filter will put all messages with a extended ID into RxFifo1
//! can.filters_extended()
//!     .push(ExtFilter::Classic {
//!         action: Action::StoreFifo1,
//!         filter: ecan::ExtendedId::MAX,
//!         mask: ecan::ExtendedId::ZERO,
//!     })
//!     .unwrap_or_else(|_| panic!("Extended filter application failed"));
//!
//! // Call to `finalize` puts MCAN into operational mode
//! let can = can.finalize().unwrap();
//!
//! // `can` object can be split into independent pieces
//! let rx_fifo_0 = can.rx_fifo_0;
//! let rx_fifo_1 = can.rx_fifo_1;
//! let tx = can.tx;
//! let tx_event_fifo = can.tx_event_fifo;
//! let aux = can.aux;
//! ```
//!
//! [`RTIC`]: https://rtic.rs
//! [`CanConfigurable`]: crate::bus::CanConfigurable
//! [`SharedMemory`]: crate::messageram::SharedMemory

pub mod bus;
pub mod config;
pub mod filter;
pub mod interrupt;
pub mod message;
pub mod messageram;
pub mod prelude;
pub mod reg;
pub mod rx_dedicated_buffers;
pub mod rx_fifo;
pub mod tx_buffers;
pub mod tx_event_fifo;

pub use embedded_can;
pub use generic_array;
pub use mcan_core as core;

// For svd2rust generated code that refers to everything via `crate::...`
use reg::generic::*;
