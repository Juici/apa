use apa::ApInt;

mod qc;

macro_rules! assert_conv {
    ($ty:ident: $($val:expr),* $(,)?) => {
        $({
            let val: $ty = $val;
            let int = ApInt::from(val);
            assert_eq!($ty::from(int), val, concat!("convert equality failed for `", stringify!($val), "`"));
        })*
    };
}

macro_rules! test_prims {
    ($($ty:ident),* $(,)?) => {
        $(
            paste::item! {
                #[test]
                fn [< from_to_ $ty >] () {
                    assert_conv!($ty: $ty::MAX, $ty::MIN);
                }

                #[test]
                fn [< prop_equivalent_from_ $ty >] () {
                    fn prop(n: $ty) -> bool {
                        n == $ty::from(ApInt::from(n))
                    }
                    qc::quickcheck(prop as fn($ty) -> bool)
                }
            }
        )*
    };
}

#[rustfmt::skip]
test_prims!(
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize,
);
