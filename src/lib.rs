//! # `pancakes`
//!
//! TODO FITZGEN

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(warnings)]

extern crate gimli;
extern crate findshlibs;

pub mod tagged_word;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[path = "./registers_x86.rs"]
pub mod registers;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
