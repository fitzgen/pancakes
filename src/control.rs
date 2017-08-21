//! Control flow for continuing or halting unwinding.

/// Whether to continue unwinding or stop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnwindControl {
    /// Continue unwinding.
    Continue,

    /// Stop unwinding.
    Break,
}

/// A trait for types that can be interpreted as an `UnwindControl`.
pub trait AsUnwindControl {
    /// Interpret this value as an `UnwindControl`.
    fn as_unwind_control(&self) -> UnwindControl {
        UnwindControl::Continue
    }
}

impl AsUnwindControl for UnwindControl {
    fn as_unwind_control(&self) -> UnwindControl {
        *self
    }
}

impl AsUnwindControl for () {
    fn as_unwind_control(&self) -> UnwindControl {
        UnwindControl::Continue
    }
}

impl<T, E> AsUnwindControl for Result<T, E> {
    fn as_unwind_control(&self) -> UnwindControl {
        if self.is_ok() {
            UnwindControl::Continue
        } else {
            UnwindControl::Break
        }
    }
}
