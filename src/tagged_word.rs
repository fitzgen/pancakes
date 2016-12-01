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

impl ops::Add for TaggedWord {
    type Output = TaggedWord;

    #[inline]
    fn add(self, rhs: TaggedWord) -> TaggedWord {
        self.and_then(|x| rhs.map(|y| x.wrapping_add(y)))
    }
}

impl ops::AddAssign for TaggedWord {
    #[inline]
    fn add_assign(&mut self, rhs: TaggedWord) {
        match (self.0, rhs.0) {
            (Some(ref mut lhs), Some(rhs)) => {
                *lhs += rhs;
            }
            (ref mut lhs, _) => {
                *lhs = None;
            }
        }
    }
}

// TODO BitXor

// TODO BitOr

// TODO BitAnd

// TODO BitAndAssign

// TODO RemAssign

// TODO Binary

// TODO ShrAssign

// TODO ShlAssign

// TODO Not

// TODO DivAssign

// TODO UpperHex

// TODO Octal

// TODO FromStr

// TODO BitXorAssign

// TODO Product

// TODO From

// TODO Rem

// TODO Div

// TODO Mul

// TODO SubAssign

// TODO Sub

// TODO Add

// TODO BitOrAssign

// TODO Sum

// TODO LowerHex

impl<T> ops::Shl<T> for TaggedWord
    where T: Copy,
          Wrapping<usize>: ops::Shl<Wrapping<T>>,
          <Wrapping<usize> as ops::Shl<Wrapping<T>>>::Output: Into<Wrapping<usize>>
{
    type Output = TaggedWord;

    #[inline]
    fn shl(self, rhs: T) -> TaggedWord {
        self.map(|x| {
            let wrapped = Wrapping(x) << Wrapping(rhs);
            wrapped.into().0
        })
    }
}

impl<T> ops::Shr<T> for TaggedWord
    where T: Copy,
Wrapping<usize>: ops::Shr<Wrapping<T>>,
<Wrapping<usize> as ops::Shr<Wrapping<T>>>::Output: Into<Wrapping<usize>>
{
    type Output = TaggedWord;

    #[inline]
    fn shr(self, rhs: T) -> TaggedWord {
        self.map(|x| {
            let wrapped = Wrapping(x) >> Wrapping(rhs);
            wrapped.into().0
        })
    }
}

impl ops::Sub for TaggedWord {
    type Output = TaggedWord;

    #[inline]
    fn sub(self, rhs: TaggedWord) -> TaggedWord {
        self.and_then(|x| rhs.map(|y| x.wrapping_sub(y)))
    }
}

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
