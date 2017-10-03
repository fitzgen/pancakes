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
mod ffi;
pub mod log;
pub mod reader;
mod tagged_word;

cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        #[path = "./x86_64/registers.rs"]
        mod registers;
    } else {
        compile_error!("Unsupported architecture; only x86_64 is currently supported");
    }
}

pub use control::{AsStackWalkControl, StackWalkControl};
pub use error::{Error, Result};
use findshlibs::{Avma, Bias, NamedMemoryRange, SectionIterable, SharedLibrary, Svma};
use gimli::UnwindSection;
pub use registers::FrameRegisters;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::ops::Range;
use std::slice;
pub use tagged_word::TaggedWord;

/// A trait for things that can read memory from the process whose stack is
/// being walked.
///
/// When we are walking our own process's stack -- perhaps to print it out
/// before panicking -- we can dereference pointers directly since we are in the
/// same address space. See `reader::ThisProcessMemory` for a `MemoryReader`
/// implementation that does this.
///
/// If we are walking a different process's stack -- perhaps we are sampling
/// stacks for an out-of-process profiler like `perf` -- then we would need to
/// use OS-specific APIs like `ptrace` or `mach` message passing to read memory
/// from that process's address space. We don't have an implementation of
/// `MemoryReader` to do this yet, but that is expected to change.
///
/// This trait can also be backed by a mock implementation during testing that
/// asserts that the expected addresses are queried, and returns deterministic
/// values.
///
/// ### Unsafety
///
/// It is the caller's responsibility to ensure that every address provided to
/// one of these methods is valid. Failure to do so will likely result in
/// dereferencing random memory.
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

/// A register set.
///
/// One could imagine multiple different `Registers` implementations for the
/// same architecture: an implementation that tracks the full set of registers,
/// that could be useful for debuggers, and an implementation that tracks the
/// subset of registers needed to perform fast-path stack walking in the 99%
/// case for profilers.
pub trait Registers: fmt::Debug + Sized {
    /// Construct this register set from the given DWARF unwind table row.
    unsafe fn from_unwind_table_row<Reader>(
        row: &gimli::UnwindTableRow<TargetEndianBuf>,
        old_registers: &Self,
        reader: &Reader,
    ) -> Result<Self>
    where
        Reader: MemoryReader;

    /// TODO FITZGEN
    fn with_current<F, T>(f: F) -> Result<T>
    where
        F: FnMut(&Self) -> Result<T>;

    /// Get the base pointer of the frame, if any.
    fn bp(&self) -> TaggedWord;

    /// Get the stack pointer.
    fn sp(&self) -> TaggedWord;

    /// Get the instruction pointer.
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

/// A configuration options builder for an `Walker`.
#[derive(Clone, Debug, Default)]
pub struct Options<'a> {
    entries: Vec<UnwindEntry<'a>>,
}

impl<'a> Options<'a> {
    /// Construct a new `Options` to start configuring stack walking options.
    ///
    /// ```
    /// use pancakes::Options;
    ///
    /// let options = Options::new();
    /// ```
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a single entry.
    pub fn add_entry(&mut self, entry: UnwindEntry<'a>) -> &mut Self {
        eprintln!(
            "FITZGEN: add_entry {:#0p} .. {:#0p}",
            entry.range.start.0 as *const (),
            entry.range.end.0 as *const (),
        );
        self.entries.push(entry);
        self
    }

    /// Add many entries.
    pub fn add_entries<I>(&mut self, entries: I) -> &mut Self
    where
        I: IntoIterator<Item = UnwindEntry<'a>>,
    {
        for entry in entries {
            self.add_entry(entry);
        }
        self
    }

    /// Create entries from the information in the given `.eh_frame` section,
    /// and add them to the builder.
    pub fn add_entries_from_eh_frame(
        &mut self,
        bias: findshlibs::Bias,
        bases: gimli::BaseAddresses,
        eh_frame: TargetEhFrame<'a>,
    ) -> Result<&mut Self> {
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
                        start: Avma(unsafe { start.offset(bias.0) }),
                        end: Avma(unsafe { start.offset(fde.len() as isize + bias.0) }),
                    };
                    self.add_entry(UnwindEntry { bias, range, fde });
                }
            }
        }
        Ok(self)
    }

    /// TODO FITZGEN
    pub fn find_eh_frame_entries(&mut self) -> Result<&mut Self> {
        cfg_if! {
            if #[cfg(target_os = "macos")] {
                const EH_FRAME: &'static [u8] = b"__eh_frame";
            } else {
                const EH_FRAME: &'static [u8] = b".eh_frame";
            }
        }

        findshlibs::TargetSharedLibrary::each(|shlib| {
            eprintln!("FITZGEN: shlib = {}", shlib.name().to_string_lossy());

            for section in shlib.sections() {
                eprintln!("FITZGEN:     section = {:?}", section.name().to_string_lossy());

                if section.name().to_bytes() == EH_FRAME {
                    let bias = shlib.virtual_memory_bias();

                    let ptr = section.actual_virtual_memory_address(shlib);
                    let ptr = ptr.0 as *const u8;
                    let len = section.len();
                    let eh_frame = unsafe {
                        slice::from_raw_parts(ptr, len)
                    };
                    let eh_frame = TargetEhFrame::new(eh_frame, gimli::NativeEndian);

                    // TODO: create base addresses properly.
                    let bases = gimli::BaseAddresses::default()
                        .set_cfi(section.stated_virtual_memory_address().0 as u64);

                    if let Err(e) = self.add_entries_from_eh_frame(bias, bases, eh_frame) {
                        // error = Some(e);
                        // return findshlibs::IterationControl::Break;

                        // TODO FITZGEN: warn or something...
                        let _ = e;
                    }

                    return findshlibs::IterationControl::Continue;
                }
            }

            findshlibs::IterationControl::Continue
        });

        Ok(self)
    }

    /// Clear all entries.
    pub fn clear_entries(&mut self) -> &mut Self {
        self.entries.clear();
        self
    }

    /// Finish configuring unwinding and create the `Walker` object with the
    /// configured options.
    pub fn build(self) -> Walker<'a> {
        self.build_with_reader_logger(reader::ThisProcessMemory, log::IgnoreLogs)
    }

    /// Finish configuring unwinding and create the `Walker` object with the
    /// configured options and the given logger.
    pub fn build_with_reader_logger<Reader, Logger>(
        mut self,
        reader: Reader,
        logger: Logger,
    ) -> Walker<'a, Reader, Logger>
    where
        Reader: MemoryReader,
        Logger: log::UnwindLogger,
    {
        self.entries.sort();
        let opts = self;
        let ctx = Some(TargetUninitializedUnwindContext::new());
        Walker {
            opts,
            reader,
            logger,
            ctx,
        }
    }
}

/// A `Walker` traverses frames that make up a native stack.
///
/// TODO FITZGEN: cache policy generic parameter?
///
/// TODO FITZGEN: make memory dereferencing operations a generic parameter
///
/// THIS WILL NOT MALLOC OR ACQUIRE LOCKS!! IT MUST BE SIGNAL SAFE!!
#[derive(Debug)]
pub struct Walker<'a, Reader = reader::ThisProcessMemory, Logger = log::IgnoreLogs>
where
    Reader: MemoryReader,
    Logger: log::UnwindLogger,
{
    opts: Options<'a>,
    reader: Reader,
    logger: Logger,
    ctx: Option<TargetUninitializedUnwindContext<'a>>,
}

impl<'a, Reader, Logger> Walker<'a, Reader, Logger>
where
    Reader: MemoryReader,
    Logger: log::UnwindLogger,
{
    /// Reconfigure this `Walker`.
    ///
    /// Turn this `Walker` back into an `Options` and perform new
    /// configuration, after which `Options::build` may be invoked to
    /// reconstruct an `Walker` with the new configuration.
    ///
    /// ```
    /// use pancakes::Options;
    ///
    /// // We have some unwinder.
    /// let unwinder = Options::new().build();
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
    pub fn reconfigure(self) -> (Options<'a>, Reader, Logger) {
        (self.opts, self.reader, self.logger)
    }

    /// Walk a single physical frame.
    unsafe fn walk_one(&mut self, start_regs: &FrameRegisters) -> Result<FrameRegisters> {
        let ip: Result<_> = start_regs.ip().into();
        let ip = ip?;

        let idx = self.opts
            .entries
            .binary_search_by(|e| {
                let ip_avma = Avma(ip as *const u8);
                eprintln!(
                    "FITZGEN: {} within {} .. {} ? {}",
                    ip_avma,
                    e.range.start,
                    e.range.end,
                    e.fde.contains(ip_avma.0.offset(-e.bias.0) as _)
                );

                if ip_avma < e.range.start {
                    eprintln!("FITZGEN:     greater");
                    Ordering::Greater
                } else if ip_avma > e.range.end {
                    eprintln!("FITZGEN:     less");
                    Ordering::Less
                } else {
                    eprintln!("FITZGEN:     equal");
                    // TODO FITZGEN: this needs to adjust for bias
                    //debug_assert!(e.fde.contains(ip_avma.0.offset(-e.bias.0) as u64));
                    Ordering::Equal
                }
            })
            .map_err(|_| Error::NoUnwindInfoForAddress(ip))?;

        let result = {
            let entry = &self.opts.entries[idx];
            eprintln!("FITZGEN: entry = {:#?}", entry);

            //let ip = (ip as *const u8).offset(-entry.bias.0);
            let ip = Avma(ip as *const u8);
            eprintln!("FITZGEN: adjusted ip = {}", ip);

            self.ctx
                .take()
                .expect("should always have Some(ctx) at the beginning of Self::walk_one")
                .initialize(entry.fde.cie())
                .map_err(|(e, ctx)| (e.into(), ctx))
                .and_then(|mut ctx| {
                    let registers = {
                        let mut table = gimli::UnwindTable::new(&mut ctx, &entry.fde);
                        loop {
                            match table.next_row() {
                                Err(e) => break Err(e.into()),
                                Ok(None) => break Ok(None),
                                Ok(Some(row)) => {
                                    let start = Svma(row.start_address() as *const u8);
                                    let end = Svma(row.end_address() as *const u8);

                                    eprintln!("FITZGEN:     row {} .. {}", start, end);

                                    let start = Avma(start.0.offset(entry.bias.0));
                                    let end = Avma(end.0.offset(entry.bias.0));

                                    if start.0 <= ip.0 && ip.0 < end.0 {
                                        eprintln!("FITZGEN:         contains!");
                                        break FrameRegisters::from_unwind_table_row(
                                            row,
                                            &start_regs,
                                            &self.reader,
                                        ).map(Some);
                                    } else {
                                        eprintln!("FITZGEN:         not contained");
                                        continue;
                                    }
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

    /// Keep walking until we've walked the whole stack, or `f` asks us to
    /// halt walking.
    ///
    /// The return value is either the `T` returned in the last invokation of
    /// `f`, on the oldest stack frame found, or the `T` where
    /// `AsStackWalkControl::as_stack_walk_control` returns
    /// `StackWalkControl::Break`.
    ///
    /// ```
    /// # fn f() {
    /// use pancakes;
    ///
    /// let mut walker = pancakes::Options::new().build();
    ///
    /// # let get_frame_regs = || unimplemented!();
    /// let result = walker.walk(get_frame_regs(), |frame| {
    ///     println!("Traversed frame {:?}", frame);
    ///
    ///     // There is an `AsStackWalkControl` implementation for `()` that
    ///     // always continues walking, so we don't need any explicit return.
    /// });
    ///
    /// // Handle walking errors however you'd like. Here, we ignore them.
    /// let _ = result;
    /// # }
    /// ```
    pub fn walk<F, T>(&mut self, start_registers: &FrameRegisters, mut f: F) -> Result<T>
    where
        F: FnMut(&FrameRegisters) -> T,
        T: AsStackWalkControl,
    {
        let mut result = f(start_registers);
        if result.as_stack_walk_control() == StackWalkControl::Break {
            return Ok(result);
        }

        let mut registers = unsafe { self.walk_one(start_registers)? };
        result = f(&registers);
        if result.as_stack_walk_control() == StackWalkControl::Break {
            return Ok(result);
        }

        loop {
            registers = unsafe { self.walk_one(start_registers)? };
            result = f(&registers);
            if result.as_stack_walk_control() == StackWalkControl::Break {
                return Ok(result);
            }
        }
    }
}
