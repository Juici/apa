use apa::ApInt;

macro_rules! test_prims {
    ($name:ident: $val:expr, $int:expr, [$($ty:ident),*]) => {
        $(
            paste::item! {
                #[test]
                fn [< $name _ $ty >] () {
                    let int: ApInt = $int;
                    let val: $ty = $val;

                    assert_eq!(val, $ty::from(&int));
                    assert_eq!(int, ApInt::from(val));
                }
            }
        )*
    };
}

test_prims!(zero: 0, ApInt::ZERO, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]);
test_prims!(one: 1, ApInt::ONE, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]);
