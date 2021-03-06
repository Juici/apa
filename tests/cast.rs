use apa::ApInt;

mod qc;

macro_rules! test_cast {
    ($from:ident as $to:ident) => {
        paste::item! {
            #[test]
            fn [< $from _as_ $to >] () {
                test_cast!($from as $to, $from::MIN);
                test_cast!($from as $to, $from::MAX);
            }
        }
    };
    ($from:ident as $to:ident, $val:expr) => {{
        let val: $from = $val;
        let int = ApInt::from(val);
        let cast = <$to>::from(int);
        assert_eq!(
            val as $to,
            cast,
            concat!(
                "cast equality failed for `",
                stringify!($val),
                " as ",
                stringify!($to),
                "`\n{:0width$b}\n{:0width$b}\n"
            ),
            val as $to,
            cast,
            width = core::mem::size_of::<$to>() * 8,
        );
    }};
}

macro_rules! test_prims {
    ($($ty:ident),* $(,)?) => {
        test_prims!(@left [$($ty)*], [$($ty)*]);
    };
    (@left [$head:ident $($tail:ident)*], [$($to:ident)*]) => {
        test_prims!(@test $head as [$($to)*]);
        test_prims!(@left [$($tail)*], [$($to)*]);
    };
    (@left [], [$($to:ident)*]) => {};
    (@test $from:ident as [$($to:ident)*]) => {
        $(
            test_cast!($from as $to);

            paste::item! {
               #[test]
               fn [< prop_equivalent_ $from _as_ $to >] () {
                    fn prop(n: $from) -> bool {
                        (n as $to) == $to::from(ApInt::from(n))
                    }
                    qc::quickcheck(prop as fn($from) -> bool)
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
