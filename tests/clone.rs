use apa::ApInt;

mod qc;

macro_rules! quickcheck_prims {
    ($($ty:ident),* $(,)*) => {
        $(
            paste::item! {
               #[test]
               fn [< prop_clone_eq_ $ty >] () {
                    fn prop(n: $ty) -> bool {
                        let i = ApInt::from(n);
                        i == i.clone()
                    }
                    qc::quickcheck(prop as fn($ty) -> bool)
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
