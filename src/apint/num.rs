use num_traits::{One, Zero};

use crate::apint::{ApInt, ApIntStorage};
use crate::limb::Limb;

impl Zero for ApInt {
    fn zero() -> Self {
        ApInt::ZERO
    }

    fn set_zero(&mut self) {
        *self = ApInt::ZERO;
    }

    fn is_zero(&self) -> bool {
        matches!(self.storage(), ApIntStorage::Stack(Limb::ZERO))
    }
}

impl One for ApInt {
    fn one() -> Self {
        ApInt::ONE
    }

    fn set_one(&mut self) {
        *self = ApInt::ONE;
    }

    fn is_one(&self) -> bool {
        matches!(self.storage(), ApIntStorage::Stack(Limb::ONE))
    }
}
