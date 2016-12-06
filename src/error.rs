//! Custom error and result types for `pancakes`.

/// TODO FITZGEN
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// TODO FITZGEN
    InvalidTaggedWord,
}

/// TODO FITZGEN
pub type Result<T> = ::std::result::Result<T, Error>;
