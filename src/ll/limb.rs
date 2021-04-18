use core::ops::Not;

// Pointer sized to allow use to use in a union with a pointer.
pub type LimbRepr = usize;

const REPR_ZERO: LimbRepr = 0x0;
const REPR_ONE: LimbRepr = 0x1;
const REPR_ONES: LimbRepr = !REPR_ZERO;

/// A part of an `Int` that fits within a single machine word.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Limb(LimbRepr);

static_assertions::assert_eq_size!(Limb, core::ptr::NonNull<Limb>);
static_assertions::const_assert!(Limb::SIZE != 0);
static_assertions::const_assert!(Limb::ALIGN != 0);

impl Limb {
    /// The size of a `Limb` in bytes.
    pub const SIZE: usize = core::mem::size_of::<Limb>();
    /// The alignment of a `Limb`.
    pub const ALIGN: usize = core::mem::align_of::<Limb>();
    /// The size of a `Limb` in bits..
    pub const BITS: usize = Self::SIZE * 8;

    /// A `Limb` with value `0`.
    pub const ZERO: Limb = Limb(REPR_ZERO);
    /// A `Limb` with value `1`.
    pub const ONE: Limb = Limb(REPR_ONE);
    /// A `Limb` with all bits set to `1`.
    pub const ONES: Limb = Limb(REPR_ONES);

    /// Returns a limb with the given value.
    #[inline(always)]
    pub const fn new(value: LimbRepr) -> Limb {
        Limb(value)
    }

    /// Returns the value of the internal representation.
    #[inline(always)]
    pub const fn repr(self) -> LimbRepr {
        self.0
    }

    /// Calculates `self` + `other`.
    ///
    /// Returns a tuple of the addition along with a boolean indicating whether
    /// an arithmetic overflow would occur. If an overflow would have occurred
    /// then the wrapped value is returned.
    #[inline(always)]
    pub const fn overflowing_add(self, other: Limb) -> (Limb, bool) {
        let (val, carry) = self.repr().overflowing_add(other.repr());
        (Limb(val), carry)
    }

    /// Calculates `self` - `other`.
    ///
    /// Returns a tuple of the subtraction along with a boolean indicating
    /// whether an arithmetic overflow would occur. If an overflow would have
    /// occurred then the wrapped value is returned.
    #[inline(always)]
    pub const fn overflowing_sub(self, other: Limb) -> (Limb, bool) {
        let (val, carry) = self.repr().overflowing_sub(other.repr());
        (Limb(val), carry)
    }

    /// Returns the number of leading zeros in the binary representation of the limb.
    #[inline(always)]
    pub const fn leading_zeros(self) -> u32 {
        self.repr().leading_zeros()
    }

    /// Returns the number of trailing zeros in the binary representation of the limb.
    #[inline(always)]
    pub const fn trailing_zeros(self) -> u32 {
        self.repr().trailing_zeros()
    }
}

impl Not for Limb {
    type Output = Limb;

    #[inline]
    fn not(self) -> Limb {
        Limb(self.repr().not())
    }
}

// Delegate formatting.
macro_rules! impl_fmt {
    ($ty:ty, [$($trait:ident),* $(,)*]) => {
        $(
            impl core::fmt::$trait for $ty {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    self.repr().fmt(f)
                }
            }
        )*
    };
}

impl_fmt!(Limb, [Binary, Octal, LowerHex, UpperHex]);
