//! Control flow for continuing or halting unwinding.

/// TODO FITZGEN
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnwindControl {
    /// TODO FITZGEN
    Continue,

    /// TODO FITZGEN
    Break,
}

/// TODO FITZGEN
pub trait AsUnwindControl {
    /// TODO FITZGEN
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
