//! # `pancakes`
//!
//! TODO FITZGEN

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(warnings)]

extern crate gimli;
extern crate findshlibs;

pub mod tagged_word;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[path = "./registers_x86.rs"]
pub mod registers;

/// TODO FITZGEN
#[derive(Debug)]
pub struct Mapping {}

/// TODO FITZGEN
#[derive(Debug)]
pub struct UnwinderBuilder {
}

impl UnwinderBuilder {
    /// Construct a new `UnwinderBuilder` to start configuring unwinding
    /// options.
    pub fn new() -> UnwinderBuilder {
        UnwinderBuilder {}
    }

    /// Add a single mapping.
    pub fn add_mapping(self, _mapping: Mapping) -> UnwinderBuilder {
        // TODO FITZGEN
        self
    }

    /// Find all the shared libraries currently loaded and add their mappings to
    /// the builder.
    pub fn add_shared_lib_mappings(self) -> UnwinderBuilder {
        // TODO FITZGEN
        self
    }

    /// Clear all mappings.
    pub fn clear_mappings(self) -> UnwinderBuilder {
        // TODO FITZGEN
        self
    }

    /// Finish configuring unwinding and create the `Unwinder` object with the
    /// configured options.
    pub fn build(self) -> Unwinder {
        Unwinder {}
    }
}

/// TODO FITZGEN
///
/// THIS WILL NOT MALLOC OR ACQUIRE LOCKS!! IT MUST BE SIGNAL SAFE!!
#[derive(Debug)]
pub struct Unwinder {
}

impl Unwinder {
    /// Reconfigure this `Unwinder`.
    ///
    /// Turn this `Unwinder` back into an `UnwinderBuilder` and perform new
    /// configuration, after which `UnwinderBuilder::build` may be invoked to
    /// reconstruct an `Unwinder` with the new configuration.
    ///
    /// ```
    /// use pancakes::UnwinderBuilder;
    ///
    /// // We have some unwinder.
    /// let unwinder = UnwinderBuilder::new().build();
    ///
    /// // Oh, we learned about new stuff to configure. Turn the unwinder back
    /// // into a builder.
    /// let builder = unwinder.reconfigure();
    /// // ... set the configuration parameters on the builder ...
    ///
    /// // Now that we are done reconfiguring, get the unwinder back again!
    /// let unwinder = builder.build();
    /// # let _ = unwinder;
    /// ```
    pub fn reconfigure(self) -> UnwinderBuilder {
        // TODO FITZGEN
        UnwinderBuilder::new()
    }

    /// Unwind a single physical frame.
    pub fn unwind_one(&self,
                      start_regs: registers::FrameUnwindRegisters)
                      -> registers::FrameUnwindRegisters {
        // TODO FITZGEN
        start_regs
    }

    /// Keep unwinding until we've walked the whole stack.
    pub fn unwind_all() {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
