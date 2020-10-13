use apa::ApInt;

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
            val as $to, cast,
            concat!(
                "cast equality failed for `",
                stringify!($val),
                " as ",
                stringify!($to),
                "`"
            )
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
        )*
    };
}

#[rustfmt::skip]
test_prims!(
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize,
);
