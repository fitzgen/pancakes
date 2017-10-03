//! Architecture specific concerns for x86 and x86_64 registers.

// TODO FITZGEN: split this into full unwinding and fast unwinding, with all
// registers vs the minimal set respectively.

use super::{Error, MemoryReader, Registers, Result, TaggedWord, TargetEndianBuf};
use ffi;
use gimli;
use std::io;
use std::mem;

// From the Sys V x86_64 ABI, figure 3.36 DWARF Register Number
// Mapping:
//
// > ...
// > General Purpose Register RBP    6    %rbp
// > Stack Pointer Register RSP      7    %rsp
// > ...
// > Return Address RA               16
// > ...
const BP: u8 = 6;
const SP: u8 = 7;
const IP: u8 = 16;

/// The registers needed to unwind a frame on x86.
#[derive(Debug)]
pub struct FrameRegisters {
    /// The `ebp`/`rbp` frame base register for this frame.
    bp: TaggedWord,

    /// The `esp`/`rsp` stack pointer register for this frame.
    sp: TaggedWord,

    /// The `eip`/`rip` instruction pointer register for this frame.
    ip: TaggedWord,
}

impl FrameRegisters {
    fn get_register(&self, register_num: u8) -> Result<TaggedWord> {
        match register_num {
            r if r == BP => Ok(self.bp),
            r if r == SP => Ok(self.sp),
            r if r == IP => Ok(self.ip),
            otherwise => Err(Error::UnknownRegister(otherwise)),
        }
    }

    unsafe fn eval_register_rule<R>(
        &self,
        rule: gimli::RegisterRule<TargetEndianBuf>,
        cfa: usize,
        reader: &R,
    ) -> TaggedWord
    where
        R: MemoryReader
    {
        match rule {
            gimli::RegisterRule::Undefined |
            gimli::RegisterRule::Architectural => TaggedWord::invalid(),

            gimli::RegisterRule::SameValue => self.bp,

            gimli::RegisterRule::Offset(offset) => reader.read_offset(cfa, offset as isize).into(),

            gimli::RegisterRule::ValOffset(offset) => TaggedWord::valid(if offset < 0 {
                cfa + (-offset as usize)
            } else {
                cfa + (offset as usize)
            }),

            gimli::RegisterRule::Register(r) => self.get_register(r).unwrap_or_default(),

            gimli::RegisterRule::Expression(_expr) => unimplemented!("TODO FITZGEN"),
            gimli::RegisterRule::ValExpression(_expr) => unimplemented!("TODO FITZGEN"),
        }
    }
}

impl Registers for FrameRegisters {
    unsafe fn from_unwind_table_row<R>(
        row: &gimli::UnwindTableRow<TargetEndianBuf>,
        old_registers: &FrameRegisters,
        reader: &R
    ) -> Result<Self>
    where
        R: MemoryReader
    {
        let cfa = match *row.cfa() {
            gimli::CfaRule::RegisterAndOffset { register, offset, } => {
                let tagged_word = old_registers.get_register(register)?;
                let word: Result<_> = tagged_word.into();
                reader.read_offset(word?, offset as isize)?
            }
            gimli::CfaRule::Expression(_expr) => unimplemented!("TODO FITZGEN"),
        };

        let bp = old_registers.eval_register_rule(row.register(BP), cfa, reader);
        let sp = old_registers.eval_register_rule(row.register(SP), cfa, reader);
        let ip = old_registers.eval_register_rule(row.register(IP), cfa, reader);

        Ok(FrameRegisters {
            bp,
            sp,
            ip,
        })
    }

    fn with_current<F, T>(mut f: F) -> Result<T>
    where
        F: FnMut(&Self) -> Result<T>
    {
        unsafe {
            let mut registers: ffi::ucontext_t = mem::zeroed();

            let r = ffi::getcontext(&mut registers);
            if r != 0 {
                return Err(Error::Io(io::Error::last_os_error()));
            }

            let registers = if cfg!(target_os = "macos") {
                assert!(!registers.uc_mcontext.is_null());
                let bp: u64 = (*registers.uc_mcontext).__ss.__rbp;
                let sp: u64 = (*registers.uc_mcontext).__ss.__rsp;
                let ip: u64 = (*registers.uc_mcontext).__ss.__rip;

                debug_assert_eq!(
                    mem::size_of::<u64>(),
                    mem::size_of::<usize>(),
                    "sanity check we didn't mess up configuration or something \
                     and aren't about to truncate registers"
                );

                FrameRegisters {
                    bp: TaggedWord::valid(bp as usize),
                    sp: TaggedWord::valid(sp as usize),
                    ip: TaggedWord::valid(ip as usize),
                }
            } else {
                unimplemented!("TODO FITZGEN")
            };

            f(&registers)
        }
    }

    fn bp(&self) -> TaggedWord { self.bp }
    fn sp(&self) -> TaggedWord { self.sp }
    fn ip(&self) -> TaggedWord { self.ip }
}
