use core::ops::{Add, Div, Mul, Neg, Rem, Sub};

use crate::apint::ApInt;

// TODO: Add implementations for core operations.

impl Add<ApInt> for ApInt {
    type Output = ApInt;

    fn add(self, _rhs: Self) -> ApInt {
        todo!()
    }
}

impl Sub<ApInt> for ApInt {
    type Output = ApInt;

    fn sub(self, _rhs: Self) -> ApInt {
        todo!()
    }
}

impl Mul<ApInt> for ApInt {
    type Output = ApInt;

    fn mul(self, _rhs: Self) -> ApInt {
        todo!()
    }
}

impl Div<ApInt> for ApInt {
    type Output = ApInt;

    fn div(self, _rhs: Self) -> ApInt {
        todo!()
    }
}

impl Rem<ApInt> for ApInt {
    type Output = ApInt;

    fn rem(self, _rhs: Self) -> ApInt {
        todo!()
    }
}

impl Neg for ApInt {
    type Output = ApInt;

    fn neg(self) -> ApInt {
        todo!()
    }
}
