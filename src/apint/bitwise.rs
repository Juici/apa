use core::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

use crate::apint::{ApInt, LimbDataMut};

impl Not for ApInt {
    type Output = ApInt;

    fn not(mut self) -> ApInt {
        match self.data_mut() {
            LimbDataMut::Stack(value) => *value = value.not(),
            LimbDataMut::Heap(ptr, len) => {
                for i in 0..len.get() {
                    // SAFETY: `i` is a valid offset from `ptr`.
                    let mut limb = unsafe { ptr.add(i) };
                    *limb = limb.not();
                }
            }
        }

        // TODO: Check valid limbs representation.

        self
    }
}

// TODO: Implement other bitwise operations.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitnot_stack() {
        let l = i32::MAX / 5;
        let r = !l;

        let l = ApInt::from(l);
        let r = ApInt::from(r);

        assert_eq!(!l, r);
    }

    #[test]
    fn bitnot_heap() {
        let l = i128::MAX / 5;
        let r = !l;

        let l = ApInt::from(l);
        let r = ApInt::from(r);

        assert_eq!(!l, r);
    }
}
