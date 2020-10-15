use crate::alloc::Vec;
use crate::apint::ApInt;

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

// Since we store data in `ApInt` in little-endian form, the string form will be reversed.
fn to_str_radix_reversed(n: &ApInt, radix: u32, upper: bool) -> Vec<u8> {
    assert!(
        2 <= radix && radix <= 36,
        "radix must be within the range 2..=36"
    );

    todo!()
}
