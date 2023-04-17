//! Module containing types and traits representing [`OwnedInterruptSet`] type
//! states
//!
//! [`OwnedInterruptSet`]: super::OwnedInterruptSet

mod private {
    /// Super trait used to mark traits with an exhaustive set of
    /// implementations
    pub trait Sealed {}
}
use private::Sealed;

// States

/// Dynamic state
///
/// State of interrupts contained in [`OIS`](super::OwnedInterruptSet) is only
/// known in runtime. A set in such a state can contain interrupts in different
/// state (disabled, enabled, etc.)
pub enum Dynamic {}
/// Disabled state
///
/// Interrupts contained in [`OIS`](super::OwnedInterruptSet) are disabled.
pub enum Disabled {}
/// Enabled on the line 0 state
///
/// Interrupts contained in [`OIS`](super::OwnedInterruptSet) are enabled on the
/// line 0.
pub enum EnabledLine0 {}
/// Enabled on the line 1 state
///
/// Interrupts contained in [`OIS`](super::OwnedInterruptSet) are enabled on the
/// line 1.
pub enum EnabledLine1 {}

// Grouping traits

/// State of interrupts contained in [`OIS`](super::OwnedInterruptSet) is known
/// in compile-time.
pub trait Static: Sealed {}
/// Interrupts contained in [`OIS`](super::OwnedInterruptSet) is _maybe_
/// enabled.
pub trait MaybeEnabled: Sealed {}

impl Sealed for Dynamic {}
impl Sealed for Disabled {}
impl Sealed for EnabledLine0 {}
impl Sealed for EnabledLine1 {}

impl Static for Disabled {}
impl Static for EnabledLine0 {}
impl Static for EnabledLine1 {}

impl MaybeEnabled for Dynamic {}
impl MaybeEnabled for EnabledLine0 {}
impl MaybeEnabled for EnabledLine1 {}
