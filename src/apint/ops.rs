use core::ops::{Add, Mul};

use crate::apint::ApInt;

// TODO: Add implementations for core operations.

impl Add<ApInt> for ApInt {
    type Output = ApInt;

    fn add(self, _rhs: Self) -> ApInt {
        todo!()
    }
}

impl Mul<ApInt> for ApInt {
    type Output = ApInt;

    fn mul(self, _rhs: Self) -> ApInt {
        todo!()
    }
}
