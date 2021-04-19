use super::*;

#[test]
fn sign() {
    assert_eq!(Int::NEG_ONE.sign(), Sign::Negative);
    assert_eq!(Int::ZERO.sign(), Sign::Zero);
    assert_eq!(Int::ONE.sign(), Sign::Positive);
}

// #[test]
// fn signum() {
//     assert_eq!(Int::NEG_ONE.signum(), Int::NEG_ONE);
//     assert_eq!(Int::ZERO.signum(), Int::ZERO);
//     assert_eq!(Int::ONE.signum(), Int::ONE);
// }
