//! Machine words that are tagged valid or invalid.

use error;
use std::mem;
use std::num::Wrapping;
use std::ops;

/// A machine word that is tagged with whether it is valid or not.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TaggedWord {
    /// A valid word.
    Valid(usize),
    /// An invalid word.
    Invalid,
}

use TaggedWord::*;

impl Default for TaggedWord {
    fn default() -> TaggedWord {
        Invalid
    }
}

impl TaggedWord {
    /// Construct a new, valid `TaggedWord`.
    #[inline]
    pub fn valid(word: usize) -> TaggedWord {
        Valid(word)
    }

    /// Construct a new, invalid `TaggedWord`.
    #[inline]
    pub fn invalid() -> TaggedWord {
        Default::default()
    }

    /// Is this tagged word valid?
    #[inline]
    pub fn is_valid(&self) -> bool {
        match *self {
            Valid(_) => true,
            Invalid => false,
        }
    }

    /// Is this tagged word invalid?
    #[inline]
    pub fn is_invalid(&self) -> bool {
        !self.is_valid()
    }

    /// If we treat this word as a pointer, is it pointing to something that is
    /// aligned on a word boundary?
    ///
    /// Invalid words are never considered word aligned.
    #[inline]
    pub fn is_word_aligned(&self) -> bool {
        self.map_or(false, |w| w & (mem::size_of::<usize>() - 1) == 0)
    }

    /// Invoke `f` on the inner word if it is valid and return a new, valid
    /// `TaggedWord` of the result. If the inner word is invalid, return an
    /// invalid `TaggedWord`.
    #[inline]
    pub fn map<F>(self, mut f: F) -> TaggedWord
    where
        F: FnMut(usize) -> usize,
    {
        match self {
            Valid(w) => Valid(f(w)),
            Invalid => Invalid,
        }
    }

    /// Invoke `f` on the inner word if it is valid and return the result. If
    /// the word is invalid, return the given default value.
    #[inline]
    pub fn map_or<T, F>(self, default: T, mut f: F) -> T
    where
        F: FnMut(usize) -> T,
    {
        match self {
            Valid(w) => f(w),
            Invalid => default,
        }
    }

    /// If the inner word is valid, invoke `f` on it and return the result. If
    /// the inner word is invalid, return another invalid word.
    #[inline]
    pub fn and_then<F>(self, mut f: F) -> TaggedWord
    where
        F: FnMut(usize) -> TaggedWord,
    {
        match self {
            Valid(word) => f(word),
            Invalid => Invalid,
        }
    }

    /// TODO FITZGEN
    pub fn unwrap_or(self, default: usize) -> usize {
        match self {
            Valid(w) => w,
            Invalid => default,
        }
    }
}

impl<E> From<Result<usize, E>> for TaggedWord {
    fn from(r: Result<usize, E>) -> TaggedWord {
        r.ok().map_or_else(TaggedWord::invalid, TaggedWord::valid)
    }
}

impl From<TaggedWord> for error::Result<usize> {
    fn from(w: TaggedWord) -> error::Result<usize> {
        match w {
            Valid(w) => Ok(w),
            Invalid => Err(error::Error::InvalidTaggedWord),
        }
    }
}

macro_rules! impl_binop {
    ( $trait_name:ident , $trait_method:ident , $x:ident , $y:ident , $imp:expr ) => {
        impl<T> ops::$trait_name<T> for TaggedWord
            where T: Into<TaggedWord>
        {
            type Output = TaggedWord;

            #[inline]
            fn $trait_method(self, rhs: T) -> TaggedWord {
                let rhs = rhs.into();
                self.and_then(|$x| rhs.map(|$y| $imp))
            }
        }
    }
}

macro_rules! impl_binop_assign {
    ( $trait_name:ident , $trait_method:ident , $x:ident , $y:ident , $imp:expr ) => {
        impl<T> ops::$trait_name<T> for TaggedWord
            where T: Into<TaggedWord>
        {
            #[inline]
            fn $trait_method(&mut self, rhs: T) {
                let rhs = rhs.into();
                match (self, rhs) {
                    (&mut Valid(ref mut $x), Valid($y)) => {
                        *$x = $imp;
                    }
                    (&mut ref mut lhs, _) => {
                        *lhs = Invalid;
                    }
                }
            }
        }
    }
}

impl_binop!(Add, add, x, y, x.wrapping_add(y));
impl_binop_assign!(AddAssign, add_assign, x, y, x.wrapping_add(y));

impl_binop!(BitAnd, bitand, x, y, x & y);
impl_binop_assign!(BitAndAssign, bitand_assign, x, y, *x & y);

impl_binop!(BitOr, bitor, x, y, x | y);
impl_binop_assign!(BitOrAssign, bitor_assign, x, y, *x | y);

impl_binop!(BitXor, bitxor, x, y, x ^ y);
impl_binop_assign!(BitXorAssign, bitxor_assign, x, y, *x ^ y);

impl_binop!(Div, div, x, y, x.wrapping_div(y));
impl_binop_assign!(DivAssign, div_assign, x, y, x.wrapping_div(y));

impl_binop!(Mul, mul, x, y, x.wrapping_mul(y));
impl_binop_assign!(MulAssign, mul_assign, x, y, x.wrapping_mul(y));

impl_binop!(Rem, rem, x, y, x.wrapping_rem(y));
impl_binop_assign!(RemAssign, rem_assign, x, y, x.wrapping_rem(y));

impl_binop!(Sub, sub, x, y, x.wrapping_sub(y));
impl_binop_assign!(SubAssign, sub_assign, x, y, x.wrapping_sub(y));

macro_rules! impl_shift {
    ( $trait_name:ident , $method:ident , $x:ident , $y:ident , $imp:expr ) => {
        impl<T> ops::$trait_name<T> for TaggedWord
            where T: Copy,
                  Wrapping<usize>: ops::$trait_name<Wrapping<T>>,
                  <Wrapping<usize> as ops::$trait_name<Wrapping<T>>>::Output: Into<Wrapping<usize>>
        {
            type Output = TaggedWord;

            #[inline]
            fn $method(self, rhs: T) -> TaggedWord {
                self.map(|x| {
                    let $x = Wrapping(x);
                    let $y = Wrapping(rhs);
                    let wrapped = $imp;
                    wrapped.into().0
                })
            }
        }
    }
}

macro_rules! impl_shift_assign {
    ( $trait_name:ident,
      $method:ident,
      $sub_trait_name:ident,
      $x:ident,
      $y:ident,
      $imp:expr ) => {
        impl<T> ops::$trait_name<T> for TaggedWord
            where T: Copy,
            Wrapping<usize>: ops::$sub_trait_name<Wrapping<T>>,
            <Wrapping<usize> as ops::$sub_trait_name<Wrapping<T>>>::Output: Into<Wrapping<usize>>
        {
            fn $method(&mut self, rhs: T) {
                *self = self.map(|x| {
                    let $x = Wrapping(x);
                    let $y = Wrapping(rhs);
                    let wrapped = $imp;
                    wrapped.into().0
                });
            }
        }
    }
}

impl_shift!(Shr, shr, x, y, x >> y);
impl_shift_assign!(ShrAssign, shr_assign, Shr, x, y, x >> y);

impl_shift!(Shl, shl, x, y, x << y);
impl_shift_assign!(ShlAssign, shl_assign, Shl, x, y, x << y);

// TODO impl Binary
// TODO impl Not
// TODO impl UpperHex
// TODO impl Octal
// TODO impl FromStr
// TODO impl Product
// TODO impl Sum
// TODO impl LowerHex

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_map_valid() {
        assert_eq!(TaggedWord::valid(5).map(|x| x + 1), TaggedWord::valid(6));
    }

    #[test]
    fn test_map_invalid() {
        assert_eq!(
            TaggedWord::invalid().map(|_| unreachable!()),
            TaggedWord::invalid()
        );
    }

    #[test]
    fn test_and_then_valid() {
        assert_eq!(
            TaggedWord::valid(5).and_then(|x| TaggedWord::valid(x + 1)),
            TaggedWord::valid(6)
        );
    }

    #[test]
    fn test_and_then_invalid() {
        assert_eq!(
            TaggedWord::invalid().and_then(|_| unreachable!()),
            TaggedWord::invalid()
        );
    }

    #[test]
    fn test_add_valid() {
        assert_eq!(
            TaggedWord::valid(1) + TaggedWord::valid(2),
            TaggedWord::valid(3)
        );
    }

    #[test]
    fn test_add_invalid() {
        assert_eq!(
            TaggedWord::valid(1) + TaggedWord::invalid(),
            TaggedWord::invalid()
        );
        assert_eq!(
            TaggedWord::invalid() + TaggedWord::valid(1),
            TaggedWord::invalid()
        );
        assert_eq!(
            TaggedWord::invalid() + TaggedWord::invalid(),
            TaggedWord::invalid()
        );
    }

    #[test]
    fn test_add_assign_valid() {
        let mut word = TaggedWord::valid(1);
        word += TaggedWord::valid(2);
        assert_eq!(word, TaggedWord::valid(3));
    }

    #[test]
    fn test_add_assign_invalid_valid() {
        let mut word = TaggedWord::invalid();
        word += TaggedWord::valid(2);
        assert_eq!(word, TaggedWord::invalid());
    }

    #[test]
    fn test_add_assign_valid_invalid() {
        let mut word = TaggedWord::valid(1);
        word += TaggedWord::invalid();
        assert_eq!(word, TaggedWord::invalid());
    }

    #[test]
    fn test_is_word_aligned() {
        assert!(TaggedWord::valid(mem::size_of::<usize>() * 1024).is_word_aligned());
        assert!(!TaggedWord::valid(1).is_word_aligned());
    }
}
