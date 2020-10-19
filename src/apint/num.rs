use core::mem;

use num_traits::{FromPrimitive, Num, NumCast, One, Signed, ToPrimitive, Zero};

use crate::apint::{ApInt, LimbData};
use crate::limb::Limb;

impl Zero for ApInt {
    fn zero() -> Self {
        ApInt::ZERO
    }

    fn set_zero(&mut self) {
        *self = ApInt::ZERO;
    }

    fn is_zero(&self) -> bool {
        matches!(self.data(), LimbData::Stack(Limb::ZERO))
    }
}

impl One for ApInt {
    fn one() -> Self {
        ApInt::ONE
    }

    fn set_one(&mut self) {
        *self = ApInt::ONE;
    }

    fn is_one(&self) -> bool {
        matches!(self.data(), LimbData::Stack(Limb::ONE))
    }
}

impl Signed for ApInt {
    fn abs(&self) -> Self {
        todo!()
    }

    fn abs_sub(&self, _other: &Self) -> Self {
        todo!()
    }

    fn signum(&self) -> Self {
        todo!()
    }

    fn is_positive(&self) -> bool {
        todo!()
    }

    fn is_negative(&self) -> bool {
        todo!()
    }
}

// TODO: Implement Num for ApInt.
impl Num for ApInt {
    type FromStrRadixErr = ();

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        todo!()
    }
}

impl FromPrimitive for ApInt {
    fn from_isize(n: isize) -> Option<ApInt> {
        Some(From::from(n))
    }

    fn from_i8(n: i8) -> Option<ApInt> {
        Some(From::from(n))
    }

    fn from_i16(n: i16) -> Option<ApInt> {
        Some(From::from(n))
    }

    fn from_i32(n: i32) -> Option<ApInt> {
        Some(From::from(n))
    }

    fn from_i64(n: i64) -> Option<ApInt> {
        Some(From::from(n))
    }

    fn from_i128(n: i128) -> Option<ApInt> {
        Some(From::from(n))
    }

    fn from_usize(n: usize) -> Option<ApInt> {
        Some(From::from(n))
    }

    fn from_u8(n: u8) -> Option<ApInt> {
        Some(From::from(n))
    }

    fn from_u16(n: u16) -> Option<ApInt> {
        Some(From::from(n))
    }

    fn from_u32(n: u32) -> Option<ApInt> {
        Some(From::from(n))
    }

    fn from_u64(n: u64) -> Option<ApInt> {
        Some(From::from(n))
    }

    fn from_u128(n: u128) -> Option<ApInt> {
        Some(From::from(n))
    }

    // FIXME: Replace from float functions with custom implementation.

    fn from_f32(n: f32) -> Option<ApInt> {
        n.to_i128().and_then(FromPrimitive::from_i128)
    }

    fn from_f64(n: f64) -> Option<ApInt> {
        n.to_i128().and_then(FromPrimitive::from_i128)
    }
}

macro_rules! to_prim {
    ($self:ident, $conv:ident) => {{
        match $self.data() {
            LimbData::Stack(value) => value.repr_signed().$conv(),
            _ => None,
        }
    }};
}

macro_rules! to_int {
    ($self:ident, $ty:ident, $conv:ident) => {{
        const LEN: usize = mem::size_of::<$ty>() / Limb::SIZE;

        match $self.data() {
            // Stack allocated int can use a direct ToPrimitive call.
            LimbData::Stack(value) => value.repr_signed().$conv(),
            // Heap allocated int requires some checks.
            LimbData::Heap(_, len) => match len.get() {
                // Fewer than or exactly `LEN` limbs.
                0..=LEN => Some(From::from($self)),
                // The int value doesn't fit within a $ty.
                _ => None,
            },
        }
    }};
}

macro_rules! to_uint {
    ($self:ident, $ty:ident, $conv:ident) => {{
        const LEN: usize = (mem::size_of::<$ty>() / Limb::SIZE) + 1;
        const LEN_M1: usize = LEN - 1;

        match $self.data() {
            // Stack allocated int can use a direct ToPrimitive call.
            LimbData::Stack(value) => value.repr_signed().$conv(),
            // Heap allocated int requires some checks.
            LimbData::Heap(ptr, len) => match len.get() {
                // Fewer than `LEN` limbs.
                0..=LEN_M1 => Some(From::from($self)),
                // Has `LEN` limbs, but last limb is zero.
                // SAFETY: This is safe since, the match guarantees
                LEN if unsafe { *ptr.add(LEN_M1) == Limb::ZERO } => Some(From::from($self)),
                // The int value doesn't fit within a $ty.
                _ => None,
            },
        }
    }};
}

impl ToPrimitive for ApInt {
    fn to_isize(&self) -> Option<isize> {
        to_prim!(self, to_isize)
    }

    fn to_i8(&self) -> Option<i8> {
        to_prim!(self, to_i8)
    }

    fn to_i16(&self) -> Option<i16> {
        to_prim!(self, to_i16)
    }

    fn to_i32(&self) -> Option<i32> {
        to_prim!(self, to_i32)
    }

    fn to_i64(&self) -> Option<i64> {
        #[cfg(target_pointer_width = "32")]
        {
            to_int!(self, i64, to_i64)
        }

        #[cfg(target_pointer_width = "64")]
        {
            to_prim!(self, to_i64)
        }
    }

    fn to_i128(&self) -> Option<i128> {
        to_int!(self, i128, to_i128)
    }

    fn to_usize(&self) -> Option<usize> {
        to_prim!(self, to_usize)
    }

    fn to_u8(&self) -> Option<u8> {
        to_prim!(self, to_u8)
    }

    fn to_u16(&self) -> Option<u16> {
        to_prim!(self, to_u16)
    }

    fn to_u32(&self) -> Option<u32> {
        #[cfg(target_pointer_width = "32")]
        {
            to_uint!(self, u32, to_u32)
        }

        #[cfg(target_pointer_width = "64")]
        {
            to_prim!(self, to_u32)
        }
    }

    fn to_u64(&self) -> Option<u64> {
        to_uint!(self, u64, to_u64)
    }

    fn to_u128(&self) -> Option<u128> {
        to_uint!(self, u128, to_u128)
    }

    // FIXME: Replace to float functions with custom implementation.

    fn to_f32(&self) -> Option<f32> {
        match self.to_i128() {
            Some(value) => value.to_f32(),
            None => self.to_u128().as_ref().and_then(ToPrimitive::to_f32),
        }
    }

    fn to_f64(&self) -> Option<f64> {
        match self.to_i128() {
            Some(value) => value.to_f64(),
            None => self.to_u128().as_ref().and_then(ToPrimitive::to_f64),
        }
    }
}

impl NumCast for ApInt {
    fn from<T: ToPrimitive>(n: T) -> Option<ApInt> {
        match n.to_i128() {
            Some(value) => FromPrimitive::from_i128(value),
            None => n.to_u128().and_then(FromPrimitive::from_u128),
        }
    }
}
