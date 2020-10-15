use apa::ApInt;
use quickcheck::{QuickCheck, StdThreadGen, Testable};

fn quickcheck<A: Testable>(f: A) {
    const N_TESTS: u64 = 10_000;

    QuickCheck::with_gen(StdThreadGen::new(usize::MAX))
        .tests(N_TESTS)
        .max_tests(N_TESTS)
        .min_tests_passed(N_TESTS)
        .quickcheck(f)
}

macro_rules! quickcheck_prims {
    ($($ty:ident),* $(,)*) => {
        quickcheck_prims!(@test [$($ty)*] [$($ty)*]);
    };
    (@test [$head:ident $($tail:ident)*] [$($ty:ident)*]) => {
        quickcheck_prims!(@convert $head);
        quickcheck_prims!(@cast $head [$($ty)*]);

        quickcheck_prims!(@test [$($tail)*] [$($ty)*]);
    };
    (@test [] [$($to:ident)*]) => {};
    (@convert $ty:ident) => {
        paste::item! {
           #[test]
           fn [< prop_equivalent_from_ $ty >] () {
                fn prop(n: $ty) -> bool {
                    n == <$ty>::from(ApInt::from(n))
                }
                quickcheck(prop as fn($ty) -> bool)
           }
        }
    };
    (@cast $from:ident [$($to:ident)*]) => {
        $(
            paste::item! {
               #[test]
               fn [< prop_equivalent_ $from _as_ $to >] () {
                    fn prop(n: $from) -> bool {
                        (n as $to) == <$to>::from(ApInt::from(n))
                    }
                    quickcheck(prop as fn($from) -> bool)
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
