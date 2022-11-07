use fugit::HertzU32 as Hz;

// TODO: Documentation
/// # Safety
/// `CanId::Address` points to valid `crate::reg::RegisterBlock`
pub unsafe trait CanId {
    const ADDRESS: *const ();
}

// TODO: Documentation
/// # Safety
/// - Clocks must not change
/// - HW register referenced by `Id: CanId` has to be owned by struct
///   implementing this trait in order to avoid aliasing.
pub unsafe trait Dependencies<Id: CanId> {
    fn host_clock(&self) -> Hz;
    fn can_clock(&self) -> Hz;
}
