#![warn(missing_docs)]

//! `mcan-core` provides a set of essential abstractions that serve as a thin
//! integration layer between platform independent [`mcan`] crate and platform
//! specific HAL crates (in documentation also referred to as _target HALs_).
//!
//! Traits from this crate are not supposed to be implemented by the
//! application developer; implementations should be provided by target HALs.
//!
//! Integrators of this crate into any given target HAL are responsible for
//! soundness of trait implementations and conforming to their respective safety
//! prerequisites.
//!
//! [`mcan`]: <https://docs.rs/crate/mcan/>

pub use fugit;

/// Trait representing CAN peripheral identity
///
/// Types implementing this trait is expected to be used as a marker type that
/// serves the purpose of identifying specific instance of CAN peripheral
/// available on the platform (as there might be more than one). It only conveys
/// *where* the CAN peripheral HW register is located, not necessarily that it
/// can be accessed. The latter is expressed by the [`Dependencies`] trait.
///
/// It is also useful for associating [`Dependencies`] with specific [`CanId`]
/// and setting up additional type constraints preventing application developers
/// from constructing a CAN abstraction with incompatible set of dependencies.
///
/// More details in [`Dependencies`] documentation.
///
/// # Safety
/// `CanId::ADDRESS` points to the start of a valid HW register of a CAN
/// peripheral
///
/// # Examples
/// ```no_run
/// use mcan_core::CanId;
///
/// pub enum Can0 {}
///
/// unsafe impl CanId for Can0 {
///     const ADDRESS: *const () = 0xDEAD0000 as *const _;
/// }
///
/// pub enum Can1 {}
///
/// unsafe impl CanId for Can1 {
///     const ADDRESS: *const () = 0xBEEF0000 as *const _;
/// }
/// ```
pub unsafe trait CanId {
    /// Static address of HW register controlling corresponding CAN peripheral
    const ADDRESS: *const ();
}

/// Trait representing CAN peripheral dependencies
///
/// Structs implementing [`Dependencies`] should
/// - enclose all object representable dependencies of [`CanId`] and release
///   them upon destruction
/// - be constructible only when it is safe and sound to interact with CAN
///   peripheral (respective clocks and pins have been already configured)
/// - be a singleton (only a single instance of [`Dependencies`] for a specific
///   [`CanId`] must exist at the same time)
///
/// in order to prevent aliasing and guarantee that high level abstractions
/// provided by [`mcan`] are sole owners of the peripheral.
///
/// Depending on a target HAL API capabilities this can assured either in
/// compile-time by type constraints or by fallible [`Dependencies`] struct
/// construction.
///
/// # Safety
/// While [`Dependencies`] type instance exists
/// - CAN related clocks must not change
/// - CAN related pins modes must not change
/// - HW register must not be safely accessible by application developer and
///   accessed in other parts of the target HAL
///
/// # Example
/// ```no_run
/// // This is just an example implementation, it is up to the `mcan`
/// // integrator how to guarantee the soundness of `mcan` usage
/// // with the target HAL API.
/// # mod pac {
/// #     pub struct CAN0;
/// #     impl CAN0 {
/// #         const PTR: *const u8 = 0xDEAD0000 as *const _;
/// #     }
/// #     pub struct CAN1;
/// #     impl CAN1 {
/// #         const PTR: *const u8 = 0xBEEF0000 as *const _;
/// #     }
/// # }
/// # mod hal {
/// #     pub mod identities {
/// #         pub enum Can0 {}
/// #         pub enum Can1 {}
/// #     }
/// # }
/// # trait PeripheralClockId {}
/// # struct PeripheralClock<ID: PeripheralClockId> {
/// #     __: core::marker::PhantomData<ID>
/// # }
/// # impl<ID: PeripheralClockId> PeripheralClock<ID> {
/// #     fn frequency(&self) -> HertzU32 {
/// #         HertzU32::from_raw(123)
/// #     }
/// # }
/// # struct HostClockToken;
/// # impl HostClockToken {
/// #     fn frequency(&self) -> HertzU32 {
/// #         HertzU32::from_raw(123)
/// #     }
/// # }
/// # struct HostClock;
/// # impl HostClock {
/// #     fn register_new_user(&mut self) -> HostClockToken { HostClockToken }
/// #     fn unregister_user(&mut self, _: HostClockToken) -> Result<(), ()> { Ok(()) }
/// # }
/// # struct Pin<ID, MODE> {
/// #     __: core::marker::PhantomData<(ID, MODE)>
/// # }
/// # struct PA22;
/// # struct PA23;
/// # struct PB12;
/// # struct PB13;
/// # struct AlternateI;
/// # struct AlternateH;
/// use fugit::HertzU32;
/// use mcan_core::CanId;
///
/// // In this example, `CanId` types are proper zero-sized marker
/// // types and one can observe that information about HW register
/// // addressing is somewhat duplicated between `pac::CAN{0, 1}`
/// // and `Can{0, 1}`.
/// //
/// // HAL design from this example assumes that a marker/identity type is
/// // reused in related contexts allowing for elaborate type constraints
/// // between abstractions from different modules (like peripheral clock
/// // for CAN and its HW register).
/// //
/// // In more classical setup, `CanId` could be just implemented by low
/// // level CAN type from PAC.
/// unsafe impl CanId for hal::identities::Can0 {
///     const ADDRESS: *const () = 0xDEAD0000 as *const _;
/// }
///
/// unsafe impl CanId for hal::identities::Can1 {
///     const ADDRESS: *const () = 0xBEEF0000 as *const _;
/// }
///
/// pub struct Dependencies<ID: PeripheralClockId, RX, TX, CAN> {
///     // This example design assumes runtime tracking of host clock
///     // users à la reference counting. `HostClock` should not be
///     // reconfigurable while having `> 0` users.
///     host_clock_token: HostClockToken,
///     // Clock object representing CAN specific asynchronous clock
///     can_peripheral_clock: PeripheralClock<ID>,
///     // Opaque field reserved for RX pin
///     rx: RX,
///     // Opaque field reserved for TX pin
///     tx: TX,
///     // Opaque field reserved for CAN HW register type (from PAC)
///     can: CAN,
/// }
///
/// impl<ID: PeripheralClockId, RX, TX, CAN> Dependencies<ID, RX, TX, CAN> {
///     // Constructor that additionally register a new user of host clock
///     pub fn new<S>(
///         host_clock: &mut HostClock,
///         can_peripheral_clock: PeripheralClock<ID>,
///         rx: RX,
///         tx: TX,
///         can: CAN,
///     ) -> Self
///     {
///         Self {
///             host_clock_token: host_clock.register_new_user(),
///             can_peripheral_clock,
///             rx,
///             tx,
///             can,
///         }
///     }
///     // Destructor that additionally unregisters from the host clock
///     pub fn free(self, host_clock: &mut HostClock) -> (PeripheralClock<ID>, RX, TX, CAN)
///     {
///         let Self {
///             host_clock_token,
///             can_peripheral_clock,
///             rx,
///             tx,
///             can,
///             ..
///         } = self;
///         host_clock.unregister_user(host_clock_token).expect("Host clock has invalid amount of users!");
///         (can_peripheral_clock, rx, tx, can)
///     }
/// }
///
/// // Trait is only implemented for valid combinations of dependencies.
/// unsafe impl<ID, RX, TX, CAN> mcan_core::Dependencies<ID> for Dependencies<ID, RX, TX, CAN>
/// where
///     ID: CanId + PeripheralClockId,
///     RX: RxPin<ValidFor = ID>,
///     TX: TxPin<ValidFor = ID>,
///     CAN: OwnedPeripheral<Represents = ID>,
/// {
///     fn host_clock(&self) -> HertzU32 {
///         self.host_clock_token.frequency()
///     }
///
///     fn can_clock(&self) -> HertzU32 {
///         self.can_peripheral_clock.frequency()
///     }
/// }
///
/// // Trait introduced in order to get 1:1 mapping from identity type to PAC type.
/// //
/// // Again, in more classical setup, `CanId` could be just implemented by low
/// // level CAN type from PAC.
/// trait OwnedPeripheral {
///     type Represents: CanId;
/// }
///
/// impl OwnedPeripheral for pac::CAN0 {
///     type Represents = hal::identities::Can0;
/// }
///
/// impl OwnedPeripheral for pac::CAN1 {
///     type Represents = hal::identities::Can1;
/// }
///
/// trait RxPin {
///     type ValidFor: CanId;
/// }
///
/// trait TxPin {
///     type ValidFor: CanId;
/// }
///
/// impl RxPin for Pin<PA23, AlternateI> {
///     type ValidFor = hal::identities::Can0;
/// }
///
/// impl TxPin for Pin<PA22, AlternateI> {
///     type ValidFor = hal::identities::Can0;
/// }
///
/// impl RxPin for Pin<PB13, AlternateH> {
///     type ValidFor = hal::identities::Can1;
/// }
///
/// impl TxPin for Pin<PB12, AlternateH> {
///     type ValidFor = hal::identities::Can1;
/// }
/// ```
/// [`mcan`]: <https://docs.rs/crate/mcan/>
pub unsafe trait Dependencies<Id: CanId> {
    /// Frequency of the host / main / CPU clock.
    ///
    /// MCAN uses CPU clock for most of its internal operations and its speed
    /// has to be equal or faster to CAN specific asynchronous clock.
    fn host_clock(&self) -> fugit::HertzU32;
    /// Frequency of CAN specific asynchronous clock.
    ///
    /// MCAN uses separate asynchronous clock for signaling / sampling and as
    /// such it should have reasonably high precision. Its speed has to be equal
    /// of slower to host clock.
    fn can_clock(&self) -> fugit::HertzU32;
}
