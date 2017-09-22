/*!

# `pancakes`

[![](http://meritbadge.herokuapp.com/pancakes)](https://crates.io/crates/pancakes) [![](https://img.shields.io/crates/d/pancakes.png)](https://crates.io/crates/pancakes) [![](https://docs.rs/pancakes/badge.svg)](https://docs.rs/pancakes/) [![Build Status](https://travis-ci.org/fitzgen/pancakes.png?branch=master)](https://travis-ci.org/fitzgen/pancakes) [![Coverage Status](https://coveralls.io/repos/github/fitzgen/pancakes/badge.svg?branch=master)](https://coveralls.io/github/fitzgen/pancakes?branch=master)

*/
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

#[macro_use]
extern crate cfg_if;
extern crate findshlibs;
extern crate gimli;

mod control;
pub mod error;
pub mod log;
pub mod reader;
mod tagged_word;

cfg_if! {
    if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        #[path = "./registers_x86.rs"]
        mod registers;
    } else {
        #[path = "./registers_unsupported.rs"]
        mod registers;
    }
}

pub use control::{AsUnwindControl, UnwindControl};
pub use error::{Error, Result};
use findshlibs::{Avma, Bias};
use gimli::UnwindSection;
pub use registers::FrameUnwindRegisters;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::ops::Range;
pub use tagged_word::TaggedWord;

/// TODO FITZGEN
pub trait MemoryReader: fmt::Debug + Sized {
    /// Read the word at the given address.
    unsafe fn read(&self, addr: usize) -> Result<usize>;

    // Provided methods.

    /// Read the word at the given offset from the given address.
    unsafe fn read_offset(&self, addr: usize, offset: isize) -> Result<usize> {
        let addr = if offset < 0 {
            addr + (-offset as usize)
        } else {
            addr + (offset as usize)
        };
        self.read(addr)
    }
}

/// TODO FITZGEN
pub trait Registers: Clone + fmt::Debug + Sized {
    /// TODO FITZGEN
    unsafe fn from_unwind_table_row<Reader>(
        row: &gimli::UnwindTableRow<TargetEndianBuf>,
        old_registers: &Self,
        reader: &Reader,
    ) -> Result<Self>
    where
        Reader: MemoryReader;

    /// TODO FITZGEN
    fn bp(&self) -> TaggedWord;

    /// TODO FITZGEN
    fn sp(&self) -> TaggedWord;

    /// TODO FITZGEN
    fn ip(&self) -> TaggedWord;
}

type TargetEndianBuf<'a> = gimli::EndianBuf<'a, gimli::NativeEndian>;
type TargetEhFrame<'a> = gimli::EhFrame<TargetEndianBuf<'a>>;
type TargetFde<'a> = gimli::FrameDescriptionEntry<TargetEhFrame<'a>, TargetEndianBuf<'a>>;
type TargetUninitializedUnwindContext<'a> = gimli::UninitializedUnwindContext<
    TargetEhFrame<'a>,
    TargetEndianBuf<'a>,
>;

/// Unwinding information for a particular address range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnwindEntry<'a> {
    range: Range<Avma>,
    bias: Bias,
    fde: TargetFde<'a>,
}

impl<'a> PartialOrd for UnwindEntry<'a> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        self.range.start.partial_cmp(&rhs.range.start)
    }
}
impl<'a> Ord for UnwindEntry<'a> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.range.start.cmp(&rhs.range.start)
    }
}

/// An configuration options builder for an `Unwinder`.
#[derive(Clone, Debug, Default)]
pub struct UnwinderOptions<'a> {
    entries: Vec<UnwindEntry<'a>>,
}

impl<'a> UnwinderOptions<'a> {
    /// Construct a new `UnwinderOptions` to start configuring unwinding
    /// options.
    ///
    /// ```
    /// use pancakes::UnwinderOptions;
    ///
    /// let options = UnwinderOptions::new();
    /// ```
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a single entry.
    pub fn add_entry(mut self, entry: UnwindEntry<'a>) -> Self {
        self.entries.push(entry);
        self
    }

    /// Add many entries.
    pub fn add_entries<I>(mut self, entries: I) -> Self
    where
        I: IntoIterator<Item = UnwindEntry<'a>>,
    {
        for entry in entries {
            self = self.add_entry(entry);
        }
        self
    }

    /// Create entries from the information in the given `.eh_frame` section,
    /// and add them to the builder.
    pub fn add_entries_from_eh_frame(
        mut self,
        bias: findshlibs::Bias,
        bases: gimli::BaseAddresses,
        eh_frame: TargetEhFrame<'a>,
    ) -> Result<Self> {
        let mut entries = eh_frame.entries(&bases);
        let mut cies = HashMap::new();
        while let Some(entry) = entries.next()? {
            match entry {
                gimli::CieOrFde::Cie(_) => continue,
                gimli::CieOrFde::Fde(partial) => {
                    let fde = partial.parse(|offset| {
                        cies.entry(offset)
                            .or_insert_with(|| eh_frame.cie_from_offset(&bases, offset))
                            .clone()
                    })?;
                    let start = fde.initial_address() as usize as *const u8;
                    let range = Range {
                        start: Avma(start),
                        end: Avma(unsafe { start.offset(fde.len() as isize + bias.0) }),
                    };
                    self = self.add_entry(UnwindEntry { bias, range, fde });
                }
            }
        }
        Ok(self)
    }

    /// Clear all entries.
    pub fn clear_entries(mut self) -> Self {
        self.entries.clear();
        self
    }

    /// Finish configuring unwinding and create the `Unwinder` object with the
    /// configured options.
    pub fn build(self) -> Unwinder<'a> {
        self.build_with_reader_logger(reader::ThisProcessMemory, log::IgnoreLogs)
    }

    /// Finish configuring unwinding and create the `Unwinder` object with the
    /// configured options and the given logger.
    pub fn build_with_reader_logger<Reader, Logger>(
        mut self,
        reader: Reader,
        logger: Logger,
    ) -> Unwinder<'a, Reader, Logger>
    where
        Reader: MemoryReader,
        Logger: log::UnwindLogger,
    {
        self.entries.sort();
        let opts = self;
        let ctx = Some(TargetUninitializedUnwindContext::new());
        Unwinder {
            opts,
            reader,
            logger,
            ctx,
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
pub struct Unwinder<'a, Reader = reader::ThisProcessMemory, Logger = log::IgnoreLogs>
where
    Reader: MemoryReader,
    Logger: log::UnwindLogger,
{
    opts: UnwinderOptions<'a>,
    reader: Reader,
    logger: Logger,
    ctx: Option<TargetUninitializedUnwindContext<'a>>,
}

impl<'a, Reader, Logger> Unwinder<'a, Reader, Logger>
where
    Reader: MemoryReader,
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
    /// let (builder, _, _) = unwinder.reconfigure();
    /// // ... set new configuration parameters on the builder ...
    ///
    /// // Now that we are done reconfiguring, get the unwinder back again!
    /// let unwinder = builder.build();
    /// # let _ = unwinder;
    /// ```
    pub fn reconfigure(self) -> (UnwinderOptions<'a>, Reader, Logger) {
        (self.opts, self.reader, self.logger)
    }

    /// Unwind a single physical frame.
    unsafe fn unwind_one(
        &mut self,
        start_regs: FrameUnwindRegisters,
    ) -> Result<FrameUnwindRegisters> {
        let ip: Result<_> = start_regs.ip().into();
        let ip = ip?;

        let idx = self.opts
            .entries
            .binary_search_by(|e| {
                let ip = ip as u64;
                if ip < e.fde.initial_address() {
                    Ordering::Less
                } else if ip > e.fde.initial_address() + e.fde.len() {
                    Ordering::Greater
                } else {
                    debug_assert!(e.fde.contains(ip));
                    Ordering::Equal
                }
            })
            .map_err(|_| Error::NoUnwindInfoForAddress(ip))?;

        let result = {
            let entry = &self.opts.entries[idx];

            self.ctx
                .take()
                .expect("should always have Some(ctx) at the beginning of Self::unwind_one")
                .initialize(entry.fde.cie())
                .map_err(|(e, ctx)| (e.into(), ctx))
                .and_then(|mut ctx| {
                    let registers = {
                        let mut table = gimli::UnwindTable::new(&mut ctx, &entry.fde);
                        loop {
                            match table.next_row() {
                                Err(e) => break Err(e.into()),
                                Ok(None) => break Ok(None),
                                Ok(Some(row)) if !row.contains(ip as u64) => continue,
                                Ok(Some(row)) => {
                                    break FrameUnwindRegisters::from_unwind_table_row(
                                        row,
                                        &start_regs,
                                        &self.reader,
                                    ).map(Some);
                                }
                            }
                        }
                    };

                    let ctx = ctx.reset();
                    match registers {
                        Ok(r) => Ok((r, ctx)),
                        Err(e) => Err((e, ctx)),
                    }
                })
        };

        match result {
            Ok((Some(registers), ctx)) => {
                self.ctx = Some(ctx);
                Ok(registers)
            }
            Ok((None, ctx)) => {
                self.ctx = Some(ctx);
                Err(Error::NoUnwindInfoForAddress(ip))
            }
            Err((e, ctx)) => {
                self.ctx = Some(ctx);
                Err(e.into())
            }
        }
    }

    /// Keep unwinding until we've walked the whole stack, or `f` asks us to
    /// halt unwinding.
    ///
    /// The return value is either the `T` returned in the last invokation of
    /// `f`, on the oldest stack frame found, or the `T` where
    /// `AsUnwindControl::as_unwind_control` returns `UnwindControl::Break`.
    ///
    /// ```
    /// # fn f() {
    /// use pancakes;
    ///
    /// let mut unwinder = pancakes::UnwinderOptions::new().build();
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
    pub fn unwind<F, T>(&mut self, _start_regs: FrameUnwindRegisters, _f: F) -> Result<T>
    where
        F: FnMut(&FrameUnwindRegisters) -> T,
        T: AsUnwindControl,
    {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
