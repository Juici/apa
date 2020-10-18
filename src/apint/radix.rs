use num_traits::Zero;

use crate::alloc::{vec, Vec};
use crate::apint::ApInt;
use crate::limb::Limb;

macro_rules! impl_fmt {
    ($trait:ident, $radix:expr, $upper:expr, $prefix:expr) => {
        impl core::fmt::$trait for ApInt {
            fn fmt(&self, _f: &mut core::fmt::Formatter) -> core::fmt::Result {
                // TODO: f.pad_integral(...)
                todo!()
            }
        }
    };
    ($trait:ident, $radix:expr, $prefix:expr) => {
        impl_fmt!($trait, $radix, false, $prefix);
    };
}

impl_fmt!(Binary, 2, "0b");
impl_fmt!(Octal, 8, "0o");
impl_fmt!(Display, 10, "");
impl_fmt!(LowerHex, 16, false, "0x");
impl_fmt!(UpperHex, 16, true, "0x");

// Based on the implementation of to_str_radix in num-bigint.
// https://github.com/rust-num/num-bigint/blob/master/src/biguint.rs

fn ilog2(v: u32) -> u8 {
    const BITS: u8 = (core::mem::size_of::<u32>() as u8) * 8;
    BITS - (v.leading_zeros() as u8) - 1
}

/// Extract little-endian bitwise digits that evenly digit `Limb`.
fn to_bitwise_digits_le(n: &ApInt, bits: u8) -> Vec<u8> {
    debug_assert!(!n.is_zero() && bits <= 8 && (Limb::BITS as u8) % bits == 0);

    todo!()
}

/// Extract little-endian bitwise digits that don't evenly digit `Limb`.
fn to_inexact_bitwise_digits_le(n: &ApInt, bits: u8) -> Vec<u8> {
    debug_assert!(!n.is_zero() && bits <= 8 && (Limb::BITS as u8) % bits != 0);

    todo!()
}

/// Extract little-endian radix digits.
#[inline(always)]
fn to_radix_digits_le(n: &ApInt, radix: u32) -> Vec<u8> {
    debug_assert!(!n.is_zero() && !radix.is_power_of_two());

    #[cfg(feature = "std")]
    let radix_log2 = f64::from(radix).log2();
    #[cfg(not(feature = "std"))]
    let radix_log2 = ilog2(radix) as f32;

    todo!()
}

fn to_radix_le(n: &ApInt, radix: u32) -> Vec<u8> {
    if n.is_zero() {
        return vec![b'0'];
    }

    match radix {
        // Powers of 2 can use bit masks and shifting instead of division.
        radix if radix.is_power_of_two() => {
            let bits = ilog2(radix);
            match (Limb::BITS as u8) % bits {
                0 => to_bitwise_digits_le(n, bits),
                _ => to_inexact_bitwise_digits_le(n, bits),
            }
        }
        // 10 is common so separate it out for const-propagation, to help
        // compiler optimisation.
        10 => to_radix_digits_le(n, 10),
        // TODO: Maybe inline 2 and 16.
        radix => to_radix_digits_le(n, radix),
    }
}

// Since we store data in `ApInt` in little-endian form, the string form will be reversed.
#[inline]
fn to_str_radix_reversed(n: &ApInt, radix: u32, upper: bool) -> Vec<u8> {
    assert!(
        2 <= radix && radix <= 36,
        "radix must be within the range 2..=36"
    );

    if n.is_zero() {
        return vec![b'0'];
    }

    // Digit values in little-endian.
    let mut vec = to_radix_le(n, radix);

    // Convert digits into bytes.
    for d in &mut vec {
        debug_assert!(u32::from(*d) < radix);

        match (*d, upper) {
            (0..=9, _) => *d += b'0',
            (_, true) => *d += b'A' - 10,
            (_, false) => *d += b'a' - 10,
        }
    }

    vec
}
