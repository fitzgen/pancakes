//! TODO FITZGEN

use super::{MemoryReader, Result};

/// TODO FITZGEN
#[derive(Debug)]
pub struct ThisProcessMemory;

impl MemoryReader for ThisProcessMemory {
    unsafe fn read(&self, addr: usize) -> Result<usize> {
        let addr = addr as *const usize;
        Ok(addr.as_ref().cloned().unwrap())
    }
}
