//! # `pancakes`
//!
//! TODO FITZGEN
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

extern crate gimli;
extern crate findshlibs;

use std::ops::Range;
use std::path::PathBuf;

pub mod control;
pub mod error;
pub mod log;
pub mod tagged_word;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[path = "./registers_x86.rs"]
pub mod registers;

/// An executable region in memory.
#[derive(Clone, Debug)]
pub struct Mapping {
    /// The file path to the executable or shared library.
    pub file_path: PathBuf,

    /// The debug information for the executable or shared library.
    pub debug_info: (),

    /// The memory range of this executable mapping.
    pub range: Range<usize>,
}

/// TODO FITZGEN
#[derive(Clone, Debug, Default)]
pub struct UnwinderOptions {
    mappings: Vec<Mapping>,
}

impl UnwinderOptions {
    /// Construct a new `UnwinderOptions` to start configuring unwinding
    /// options.
    pub fn new() -> UnwinderOptions {
        Default::default()
    }

    /// Add a single mapping.
    pub fn add_mapping(mut self, mapping: Mapping) -> UnwinderOptions {
        self.mappings.push(mapping);
        self
    }

    /// Add many mappings.
    pub fn add_mappings<I>(mut self, mappings: I) -> UnwinderOptions
    where
        I: IntoIterator<Item = Mapping>,
    {
        for mapping in mappings {
            self = self.add_mapping(mapping);
        }
        self
    }

    /// Find the main executable and all the shared libraries currently loaded
    /// and add their mappings to the builder.
    pub fn discover_mappings(self) -> UnwinderOptions {
        // TODO FITZGEN
        self
    }

    /// Clear all mappings.
    pub fn clear_mappings(mut self) -> UnwinderOptions {
        self.mappings.clear();
        self
    }

    /// Finish configuring unwinding and create the `Unwinder` object with the
    /// configured options.
    pub fn build(self) -> Unwinder {
        Unwinder {
            opts: self,
            logger: log::IgnoreLogs,
        }
    }

    /// Finish configuring unwinding and create the `Unwinder` object with the
    /// configured options and the given logger.
    pub fn build_with_logger<Logger>(self, logger: Logger) -> Unwinder<Logger>
    where
        Logger: log::UnwindLogger,
    {
        Unwinder {
            opts: self,
            logger: logger,
        }
    }
}

/// TODO FITZGEN
///
/// TODO FITZGEN: cache policy generic parameter?
///
/// TODO FITZGEN: make memory dereferencing operations a generic parameter
///
/// THIS WILL NOT MALLOC OR ACQUIRE LOCKS!! IT MUST BE SIGNAL SAFE!!
#[derive(Debug)]
pub struct Unwinder<Logger = log::IgnoreLogs>
where
    Logger: log::UnwindLogger,
{
    opts: UnwinderOptions,
    logger: Logger,
}

impl<Logger> Unwinder<Logger>
where
    Logger: log::UnwindLogger,
{
    /// Reconfigure this `Unwinder`.
    ///
    /// Turn this `Unwinder` back into an `UnwinderOptions` and perform new
    /// configuration, after which `UnwinderOptions::build` may be invoked to
    /// reconstruct an `Unwinder` with the new configuration.
    ///
    /// ```
    /// use pancakes::UnwinderOptions;
    ///
    /// // We have some unwinder.
    /// let unwinder = UnwinderOptions::new().build();
    ///
    /// // Oh, we learned about new stuff to configure. Turn the unwinder back
    /// // into a builder.
    /// let (builder, _) = unwinder.reconfigure();
    /// // ... set new configuration parameters on the builder ...
    ///
    /// // Now that we are done reconfiguring, get the unwinder back again!
    /// let unwinder = builder.build();
    /// # let _ = unwinder;
    /// ```
    pub fn reconfigure(self) -> (UnwinderOptions, Logger) {
        (self.opts, self.logger)
    }

    /// Unwind a single physical frame.
    pub fn unwind_one(
        &self,
        start_regs: registers::FrameUnwindRegisters,
    ) -> error::Result<registers::FrameUnwindRegisters> {
        // TODO FITZGEN
        Ok(start_regs)
    }

    /// Keep unwinding until we've walked the whole stack, or `f` asks us to
    /// halt unwinding.
    ///
    /// The return value is either the `T` returned in the last invokation of
    /// `f`, on the oldest stack frame found, or the `T` where
    /// `AsUnwindControl::as_unwind_control` returns `AsUnwindControl::Break`.
    ///
    /// ```
    /// # fn f() {
    /// use pancakes;
    ///
    /// let unwinder = pancakes::UnwinderOptions::new().build();
    ///
    /// # let get_frame_regs = || unimplemented!();
    /// let result = unwinder.unwind(get_frame_regs(), |frame| {
    ///     println!("Unwound frame {:?}", frame);
    ///
    ///     // There is an `AsUnwindControl` implementation for `()` that always
    ///     // continues unwinding, so we don't need any explicit return.
    /// });
    ///
    /// // Handle unwinding errors however you'd like. Here, we ignore them.
    /// let _ = result;
    /// # }
    /// ```
    pub fn unwind<F, T>(
        &self,
        _start_regs: registers::FrameUnwindRegisters,
        _f: F,
    ) -> error::Result<T>
    where
        F: FnMut(&registers::FrameUnwindRegisters) -> T,
        T: control::AsUnwindControl,
    {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
