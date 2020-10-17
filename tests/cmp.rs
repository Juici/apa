use apa::ApInt;

mod qc;

macro_rules! quickcheck_prims {
    ($($ty:ident),* $(,)*) => {
        $(
            paste::item! {
               #[test]
               fn [< prop_cmp_ $ty >] () {
                    fn prop(l: $ty, r: $ty) -> bool {
                        let li = ApInt::from(l);
                        let ri = ApInt::from(r);

                        l.cmp(&r) == li.cmp(&ri)
                    }
                    qc::quickcheck(prop as fn($ty, $ty) -> bool)
               }
            }
        )*
    };
}

#[rustfmt::skip]
quickcheck_prims!(
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize,
);
