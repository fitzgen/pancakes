//! Control flow for continuing or halting stack walking.

/// Whether to continue unwinding or stop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StackWalkControl {
    /// Continue unwinding.
    Continue,

    /// Stop unwinding.
    Break,
}

/// A trait for types that can be interpreted as an `StackWalkControl`.
pub trait AsStackWalkControl {
    /// Interpret this value as an `StackWalkControl`.
    fn as_stack_walk_control(&self) -> StackWalkControl {
        StackWalkControl::Continue
    }
}

impl AsStackWalkControl for StackWalkControl {
    fn as_stack_walk_control(&self) -> StackWalkControl {
        *self
    }
}

impl AsStackWalkControl for () {
    fn as_stack_walk_control(&self) -> StackWalkControl {
        StackWalkControl::Continue
    }
}

impl<T, E> AsStackWalkControl for Result<T, E> {
    fn as_stack_walk_control(&self) -> StackWalkControl {
        if self.is_ok() {
            StackWalkControl::Continue
        } else {
            StackWalkControl::Break
        }
    }
}
