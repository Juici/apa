mod repr;

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
    pub const ZERO: Int = Int::from_limb(Limb::ZERO);
    /// The multiplicative identity, `1`.
    pub const ONE: Int = Int::from_limb(Limb::ONE);

    /// Returns an [`Int`] with inlined value `n`.
    pub const fn from_usize(n: usize) -> Int {
        Int::from_limb(Limb::new(n))
    }

    /// Returns an [`Int`] with inlined value `n`.
    pub const fn from_isize(n: isize) -> Int {
        let len = match n {
            n if n > 0 => ReprLen(1),
            0 => ReprLen(0),
            _ => ReprLen(-1),
        };

        let limb = Limb::new(n.unsigned_abs());
        let repr = Repr { inline: limb };

        Int { repr, len }
    }
}
