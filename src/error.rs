//! Custom error and result types for `pancakes`.

use gimli;
use std::error::Error as ErrorTrait;
use std::fmt;
use std::io;

/// The different kinds of errors that can occur when walking a stack.
pub enum Error {
    /// An IO error.
    Io(io::Error),

    /// An error parsing debug information with `gimli`.
    Gimli(gimli::Error),

    /// Expected a valid word, but found an invalid one.
    InvalidTaggedWord,

    /// There is no unwinding information for the given address.
    NoUnwindInfoForAddress(usize),

    /// An unknown DWARF register number.
    UnknownRegister(u8),
}
use Error::*;

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error(")?;
        fmt::Display::fmt(self, f)?;
        write!(f, ")")
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Io(ref e) => write!(f, "{}", e),
            Gimli(ref e) => write!(f, "Error parsing debug info: {}", e),
            InvalidTaggedWord => write!(f, "{}", self.description()),
            NoUnwindInfoForAddress(addr) => write!(f, "No unwind information for {:#x}", addr),
            UnknownRegister(reg) => write!(f, "Unknown DWARF register number: {}", reg),
        }
    }
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        match *self {
            Io(ref e) => e.description(),
            Gimli(_) => "Error parsing debug info",
            InvalidTaggedWord => "Invalid tagged word",
            NoUnwindInfoForAddress(_) => {
                "Tried to walk across a frame we do not have unwind information for"
            }
            UnknownRegister(_) => "Unknown DWARF register number",
        }
    }

    fn cause(&self) -> Option<&ErrorTrait> {
        match *self {
            Io(ref e) => Some(e),
            Gimli(ref e) => Some(e),
            InvalidTaggedWord | NoUnwindInfoForAddress(_) | UnknownRegister(_) => None,
        }
    }
}

impl From<gimli::Error> for Error {
    fn from(g: gimli::Error) -> Error {
        Error::Gimli(g)
    }
}

/// Either a `T` or a `pancakes::Error`.
pub type Result<T> = ::std::result::Result<T, Error>;
