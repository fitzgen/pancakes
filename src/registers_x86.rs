//! Architecture specific concerns for x86 and x86_64 registers.

use tagged_word;

/// The registers needed to unwind a frame on x86.
#[derive(Clone, Copy, Debug)]
pub struct FrameUnwindRegisters {
    /// The `ebp`/`rbp` frame base register for this frame.
    pub bp: tagged_word::TaggedWord,

    /// The `esp`/`rsp` stack pointer register for this frame.
    pub sp: tagged_word::TaggedWord,

    /// The `eip`/`rip` instruction pointer register for this frame.
    pub ip: tagged_word::TaggedWord,
}
