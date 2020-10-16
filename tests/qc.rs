use quickcheck::{QuickCheck, StdThreadGen, Testable};

pub fn quickcheck<A: Testable>(f: A) {
    const N_TESTS: u64 = 10_000;

    QuickCheck::with_gen(StdThreadGen::new(usize::MAX))
        .tests(N_TESTS)
        .max_tests(N_TESTS)
        .min_tests_passed(N_TESTS)
        .quickcheck(f)
}
