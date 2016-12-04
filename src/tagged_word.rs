//! Machine words that are tagged valid or invalid.

use std::num::Wrapping;
use std::ops;

/// A machine word that is tagged with whether it is valid or not.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TaggedWord(Option<usize>);

impl Default for TaggedWord {
    fn default() -> TaggedWord {
        TaggedWord(None)
    }
}

impl TaggedWord {
    /// Construct a new, valid `TaggedWord`.
    #[inline]
    pub fn valid(word: usize) -> TaggedWord {
        TaggedWord(Some(word))
    }

    /// Construct a new, invalid `TaggedWord`.
    #[inline]
    pub fn invalid() -> TaggedWord {
        Default::default()
    }

    /// Is this tagged word valid?
    pub fn is_valid(&self) -> bool {
        self.0.is_some()
    }

    /// Is this tagged word invalid?
    pub fn is_invalid(&self) -> bool {
        !self.is_valid()
    }

    /// Invoke `f` on the inner word if it is valid and return a new, valid
    /// `TaggedWord` of the result. If the inner word is invalid, return an
    /// invalid `TaggedWord`.
    #[inline]
    pub fn map<F>(self, f: F) -> TaggedWord
        where F: FnMut(usize) -> usize
    {
        TaggedWord(self.0.map(f))
    }

    /// If the inner word is valid, invoke `f` on it and return the result. If
    /// the inner word is invalid, return another invalid word.
    #[inline]
    pub fn and_then<F>(self, mut f: F) -> TaggedWord
        where F: FnMut(usize) -> TaggedWord
    {
        match self.0 {
            Some(word) => f(word),
            None => TaggedWord(None),
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
                match (self.0, rhs.0) {
                    (Some(ref mut $x), Some($y)) => {
                        *$x = $imp;
                    }
                    (ref mut lhs, _) => {
                        *lhs = None;
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
// TODO impl From
// TODO impl Sum
// TODO impl LowerHex

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_valid() {
        assert_eq!(TaggedWord::valid(5).map(|x| x + 1), TaggedWord::valid(6));
    }

    #[test]
    fn test_map_invalid() {
        assert_eq!(TaggedWord::invalid().map(|_| unreachable!()),
                   TaggedWord::invalid());
    }

    #[test]
    fn test_and_then_valid() {
        assert_eq!(TaggedWord::valid(5).and_then(|x| TaggedWord::valid(x + 1)),
                   TaggedWord::valid(6));
    }

    #[test]
    fn test_and_then_invalid() {
        assert_eq!(TaggedWord::invalid().and_then(|_| unreachable!()),
                   TaggedWord::invalid());
    }

    #[test]
    fn test_add_valid() {
        assert_eq!(TaggedWord::valid(1) + TaggedWord::valid(2),
                   TaggedWord::valid(3));
    }

    #[test]
    fn test_add_invalid() {
        assert_eq!(TaggedWord::valid(1) + TaggedWord::invalid(),
                   TaggedWord::invalid());
        assert_eq!(TaggedWord::invalid() + TaggedWord::valid(1),
                   TaggedWord::invalid());
        assert_eq!(TaggedWord::invalid() + TaggedWord::invalid(),
                   TaggedWord::invalid());
    }
}
