//! The definition and implementations of `UnwindLogger`.

use std::fmt::Debug;
use std::io::{self, Write};

macro_rules! log {
    ( $ logger : expr , $ fmt : expr ) => {
        use ::std::io::Write;
        let _ = writeln!($logger, $fmt);
    };
    ( $ logger : expr , $ fmt : expr , $ ( $ arg : tt ) * ) => {
        use ::std::io::Write;
        let _ = writeln!($logger, $fmt, $($arg)*);
    };
}

/// TODO FITZGEN
pub trait UnwindLogger: Debug + Write + Sized {}

/// TODO FITZGEN
#[derive(Debug)]
pub struct IgnoreLogs;

impl Write for IgnoreLogs {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<T> UnwindLogger for T
where
    T: Debug + Write + Sized,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_macro() {
        let mut logger = IgnoreLogs;
        log!(&mut logger, "Wow! {}", 42);
    }
}
