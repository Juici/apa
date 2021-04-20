#[cfg(test)]
mod tests;

pub(crate) mod repr;

use crate::ll;
use crate::ll::limb::Limb;

use self::repr::{Repr, ReprLen};

/// The sign of a number.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(i8)]
pub enum Sign {
    /// Negative number.
    Negative = -1,
    /// Zero.
    Zero = 0,
    /// Positive number.
    Positive = 1,
}

/// An arbitrary-precision integer.
pub struct Int {
    repr: Repr,
    len: ReprLen,
}

impl Int {
    /// The additive identity, `0`.
    pub const ZERO: Int = Int::from_isize(0);
    /// The multiplicative identity, `1`.
    pub const ONE: Int = Int::from_isize(1);

    /// The additive inverse of [`ONE`][Self::ONE], `-1`.
    pub const NEG_ONE: Int = Int::from_isize(-1);

    /// Returns an [`Int`] with inlined value `n`.
    #[inline]
    pub const fn from_usize(n: usize) -> Int {
        Int::from_limb(Limb::new(n))
    }

    /// Returns an [`Int`] with inlined value `n`.
    #[inline]
    pub const fn from_isize(n: isize) -> Int {
        let len = match n {
            n if n > 0 => ReprLen::new(1),
            0 => ReprLen::new(0),
            _ => ReprLen::new(-1),
        };

        let limb = Limb::new(n.unsigned_abs());
        let repr = Repr { inline: limb };

        Int { repr, len }
    }

    /// Returns the [`Sign`] of this integer.
    #[inline(always)]
    pub const fn sign(&self) -> Sign {
        self.len.sign()
    }

    /// Returns an integer representing the sign of `self`.
    /// - `-1` if `self` is negative.
    /// - `0` if `self` is zero.
    /// - `1` if `self` is positive.
    #[inline(always)]
    pub const fn signum(&self) -> Int {
        match self.sign() {
            Sign::Negative => Int::NEG_ONE,
            Sign::Zero => Int::ZERO,
            Sign::Positive => Int::ONE,
        }
    }

    /// Consumes `self` and returns its absolute value.
    #[inline]
    pub const fn abs(mut self) -> Int {
        self.len = self.len.abs();
        self
    }
}

impl PartialEq<usize> for Int {
    #[inline]
    fn eq(&self, other: &usize) -> bool {
        // Only zero or positive single limb integers can match.
        if !matches!(self.len.repr(), 0 | 1) {
            return false;
        }
        // SAFETY: Representation is inline.
        unsafe { self.repr.inline.repr() == *other }
    }
}

impl PartialEq<isize> for Int {
    #[inline]
    fn eq(&self, other: &isize) -> bool {
        // The signum of `other` is guaranteed to be one of -1, 0, or 1.
        let signum = other.signum() as i32;

        // If `len` matches `signum`, then we know that `self` has the same
        // sign as `other`, and that the representation of `self` is inline.
        if self.len.repr() != signum {
            // Only single limb integers can match.
            return false;
        }

        // At this point we know that `self` and `other` have the same sign,
        // we now only care that their absolute values match.
        // SAFETY: Representation is inline.
        unsafe { self.repr.inline.repr() == other.unsigned_abs() }
    }
}

impl PartialEq for Int {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        if self.len.is_inline() {
            // SAFETY: Representation of `self` and `other` is inline.
            unsafe { self.repr.inline == other.repr.inline }
        } else {
            // SAFETY: `self` and `other` both have length `self.len`.
            unsafe { ll::eq(self.as_ptr(), other.as_ptr(), self.len.len()) }
        }
    }
}

impl Eq for Int {}
