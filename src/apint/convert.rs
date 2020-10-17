use core::mem::MaybeUninit;
use core::num::NonZeroUsize;

use crate::apint::{ApInt, LimbData};
use crate::limb::{Limb, LimbRepr};

macro_rules! impl_from_prim {
    (unsigned: $($ty:ident),* $(,)?) => {
        $(
            impl core::convert::From<$ty> for ApInt {
                fn from(val: $ty) -> ApInt {
                    const SIZE_TY: usize = core::mem::size_of::<$ty>();
                    const SIZE_LIMB: usize = Limb::SIZE;

                    const BITS_TY: usize = SIZE_TY * 8;
                    const BITS_LIMB: usize = Limb::BITS;

                    const FITS: bool = SIZE_TY < SIZE_LIMB;

                    // The number of bits actually required to hold the value.
                    let bits_val = BITS_TY - (val.leading_zeros() as usize);

                    // Check if the value fits, or can be truncated to fit.
                    if FITS || bits_val < BITS_LIMB {
                        ApInt::from_limb(Limb(val as LimbRepr))
                    } else {
                        // Equivalent to `ceil((bits_val + 1) / BITS_LIMB)`.
                        let capacity = (bits_val / BITS_LIMB) + 1;
                        // SAFETY: `factor + 1` is guaranteed to be greater than 1.
                        let capacity = unsafe { NonZeroUsize::new_unchecked(capacity) };

                        let mut int = ApInt::with_capacity(capacity);

                        let val = val.to_le();
                        // SAFETY: This is safe since we are copying as many bytes as the smaller of
                        //         the size of the value type or the capacity of the ApInt in bytes.
                        unsafe {
                            core::ptr::copy_nonoverlapping(
                                &val as *const $ty as *const u8,
                                int.limbs_mut().as_ptr() as *mut u8,
                                SIZE_TY.min(capacity.get() * SIZE_LIMB),
                            );
                        }

                        int
                    }
                }
            }
        )*
    };
    (signed: $($ty:ty),* $(,)?) => {
        $(
            impl core::convert::From<$ty> for ApInt {
                fn from(val: $ty) -> ApInt {
                    const SIZE_TY: usize = core::mem::size_of::<$ty>();
                    const SIZE_LIMB: usize = Limb::SIZE;

                    const BITS_TY: usize = SIZE_TY * 8;
                    const BITS_LIMB: usize = Limb::BITS;

                    const FITS: bool = SIZE_TY < SIZE_LIMB;

                    const SHIFT_TY: usize = BITS_TY - 1;
                    const SIGN_TY: $ty = 1 << SHIFT_TY;

                    let abs_val = val & !SIGN_TY;
                    let leading = (val.leading_zeros() + val.leading_ones()) as usize;

                    // The number of bits actually required to hold the absolute value plus
                    // an additional sign bit.
                    let bits_val = BITS_TY - leading + 1;

                    // Check if the value fits, or can be truncated to fit.
                    if FITS || bits_val <= BITS_LIMB {
                        // Apply sign bit to limb.
                        let sign_limb = (val & SIGN_TY) as LimbRepr;
                        let limb = (abs_val as LimbRepr) | sign_limb;

                        ApInt::from_limb(Limb(limb))
                    } else {
                        // Equivalent to `ceil(bits_val / BITS_LIMB)`.
                        let capacity = {
                            let q = bits_val / BITS_LIMB;
                            let r = bits_val % BITS_LIMB;
                            q + ((r != 0) as usize)
                        };
                        // SAFETY: `factor` is guaranteed to be greater than 1,
                        //          since `bits_val` >= `BITS_LIMB`.
                        let capacity = unsafe { NonZeroUsize::new_unchecked(capacity) };

                        let mut int = ApInt::with_capacity(capacity);

                        let val = val.to_le();
                        // SAFETY: This is safe since we are copying as many bytes as the smaller of
                        //         the size of the value type or the capacity of the ApInt in bytes.
                        unsafe {
                            core::ptr::copy_nonoverlapping(
                                &val as *const $ty as *const u8,
                                int.limbs_mut().as_ptr() as *mut u8,
                                SIZE_TY.min(capacity.get() * SIZE_LIMB),
                            );
                        }

                        int
                    }
                }
            }
        )*
    };
}

impl_from_prim!(unsigned: u8, u16, u32, u64, u128, usize);
impl_from_prim!(signed: i8, i16, i32, i64, i128, isize);

macro_rules! impl_to_prim {
    ($($ty:ident),* $(,)?) => {
        $(
            impl<'a> core::convert::From<&'a ApInt> for $ty {
                fn from(int: &'a ApInt) -> $ty {
                    const SIZE_TY: usize = core::mem::size_of::<$ty>();
                    const SIZE_LIMB: usize = Limb::SIZE;
                    const BITS_LIMB: usize = Limb::BITS;
                    const SHIFT_LIMB: usize = BITS_LIMB - 1;

                    unsafe {
                        match int.data() {
                            LimbData::Stack(limb) => limb.repr_signed() as $ty,
                            LimbData::Heap(ptr) => match SIZE_LIMB * int.len.get() {
                                size_int if SIZE_TY <= size_int => $ty::from_le(*ptr.as_ptr().cast()),
                                _ => {
                                    // The number of limbs that can fit in $t.
                                    const FACTOR: usize = SIZE_TY / SIZE_LIMB;
                                    // Copy as many limbs as we have or that can fit in $t.
                                    let n_copy = int.len.get().min(FACTOR);

                                    // Last limb has the sign.
                                    let sign_limb = (*ptr.add(int.len.get() - 1)).repr_signed();
                                    // Propagate the sign across the limb, taking advantage of signed shift.
                                    let sign_byte = (sign_limb >> SHIFT_LIMB) as u8;

                                    let mut val = MaybeUninit::uninit();
                                    // Initialise with sign bits.
                                    core::ptr::write_bytes(val.as_mut_ptr(), sign_byte, 1);
                                    core::ptr::copy_nonoverlapping(ptr.as_ptr(), val.as_mut_ptr() as *mut Limb, n_copy);
                                    $ty::from_le(val.assume_init())
                                }
                            },
                        }
                    }
                }
            }

            impl core::convert::From<ApInt> for $ty {
                #[inline]
                fn from(int: ApInt) -> $ty {
                    $ty::from(&int)
                }
            }
        )*
    };
}

impl_to_prim!(u8, u16, u32, u64, u128, usize);
impl_to_prim!(i8, i16, i32, i64, i128, isize);
