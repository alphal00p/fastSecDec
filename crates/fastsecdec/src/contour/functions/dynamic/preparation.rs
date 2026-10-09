//! Target precision is a preparation-time property, never a sampling input.
use std::cell::Cell;

thread_local! {
    static PRECISION: Cell<Option<u32>> = const { Cell::new(None) };
}

/// Symbolica currently passes tags, but not mapping precision, to callback
/// factories. Its native coefficient mapping resolves those factories inline.
/// Preserve an outer preparation when nested mapping or unwinding occurs.
pub(crate) fn with_precision<R>(bits: u32, prepare: impl FnOnce() -> R) -> R {
    struct Reset(Option<u32>);
    impl Drop for Reset {
        fn drop(&mut self) {
            PRECISION.set(self.0);
        }
    }
    let _reset = Reset(PRECISION.replace(Some(bits)));
    prepare()
}

pub(super) fn precision() -> Option<u32> {
    PRECISION.get()
}
