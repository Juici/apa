use apa::ApInt;
use num_traits::{One, Zero};

#[test]
fn zero() {
    assert!(ApInt::ZERO.is_zero());
}

#[test]
fn one() {
    assert!(ApInt::ONE.is_one());
}
