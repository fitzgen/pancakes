//! No-op registers for unsupported platforms.

use super::{Registers, TaggedWord};

/// An empty register set.
#[derive(Clone, Copy, Debug)]
pub struct FrameUnwindRegisters;

impl Registers for FrameUnwindRegisters {
    fn bp(&self) -> TaggedWord { TaggedWord::invalid() }
    fn sp(&self) -> TaggedWord { TaggedWord::invalid() }
    fn ip(&self) -> TaggedWord { TaggedWord::invalid() }
}
