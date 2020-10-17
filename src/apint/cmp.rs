use core::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};

use crate::apint::{ApInt, LimbData};
use crate::limb::Limb;

impl PartialEq for ApInt {
    fn eq(&self, other: &Self) -> bool {
        match (self.data(), other.data()) {
            // Compare stack values.
            (LimbData::Stack(l), LimbData::Stack(r)) => l == r,
            // Compare heap limbs.
            (LimbData::Heap(l_ptr), LimbData::Heap(r_ptr)) if self.len == other.len => {
                let mut i = (self.len.get() - 1) as isize;
                // No need to check at start of loop, since `len - 1 >= 0` is
                // guaranteed.
                loop {
                    // At this point `i >= 0` is guaranteed, so casting to
                    // usize will not overflow.

                    // SAFETY: `i` is within the bounds of `l_ptr`.
                    let l = unsafe { *l_ptr.add(i as usize) };
                    // SAFETY: `i` is within the bounds of `r_ptr`.
                    let r = unsafe { *r_ptr.add(i as usize) };
                    if l != r {
                        return false;
                    }

                    i -= 1;
                    // Return true if all limbs have been compared.
                    // No need to check `i < 0`, since we step in intervals of `-1`.
                    if i == -1 {
                        return true;
                    }
                }
            }
            // Different representations or lengths.
            _ => false,
        }
    }
}

impl Eq for ApInt {}

impl PartialOrd for ApInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ApInt {
    fn cmp(&self, other: &Self) -> Ordering {
        const SHIFT: usize = Limb::BITS - 1;

        match (self.data(), other.data()) {
            // Compare stack values.
            (LimbData::Stack(l), LimbData::Stack(r)) => l.repr_signed().cmp(&r.repr_signed()),
            // Compare heap limbs.
            (LimbData::Heap(l_ptr), LimbData::Heap(r_ptr)) => {
                // SAFETY: `i` is within the bounds of `l_ptr`.
                let l = unsafe { *l_ptr.add(self.len.get() - 1) };
                // SAFETY: `i` is within the bounds of `r_ptr`.
                let r = unsafe { *r_ptr.add(other.len.get() - 1) };

                // Compare sign bits.
                let l_bit = l.repr_ne() >> SHIFT;
                let r_bit = r.repr_ne() >> SHIFT;

                match (l_bit, r_bit) {
                    (0, 1) => return Ordering::Greater,
                    (1, 0) => return Ordering::Less,
                    _ => {}
                }

                // Same sign bits, compare number of limbs.
                match self.len.cmp(&other.len) {
                    Ordering::Equal => {}
                    // Positive sign bit.
                    ordering if l_bit == 0 => return ordering,
                    // Negative sign bit.
                    ordering => return ordering.reverse(),
                }

                // At this point it is guaranteed that both ints have the same
                // number of limbs.
                //
                // Sign doesn't matter anymore and we can
                // compare each limb as unsigned values, due to how numbers are
                // represented in two's complement.

                let mut i = (self.len.get() - 1) as isize;

                // No need to check at start of loop, since `i >= 0` is
                // guaranteed.
                loop {
                    // At this point `i >= 0` is guaranteed, so casting to
                    // usize will not overflow.

                    // SAFETY: `i` is within the bounds of `l_ptr`.
                    let l = unsafe { *l_ptr.add(i as usize) };
                    // SAFETY: `i` is within the bounds of `r_ptr`.
                    let r = unsafe { *r_ptr.add(i as usize) };
                    match l.repr_ne().cmp(&r.repr_ne()) {
                        Ordering::Equal => {}
                        ordering => return ordering,
                    }

                    i -= 1;
                    // Return true if all limbs have been compared.
                    // No need to check `i < 0`, since we step in intervals of `-1`.
                    if i == -1 {
                        return Ordering::Equal;
                    }
                }
            }
            // Different representations.
            (LimbData::Stack(_l), LimbData::Heap(r_ptr)) => {
                // SAFETY: `len - 1` is within the bounds of `r_ptr`.
                let r = unsafe { *r_ptr.add(other.len.get() - 1) };

                // The heap value has a larger absolute value, so check its
                // sign bit.
                let r_bit = r.repr_ne() >> SHIFT;

                match r_bit {
                    // Heap value is negative.
                    1 => Ordering::Greater,
                    // Heap value is positive.
                    _ => Ordering::Less,
                }
            }
            // Different representations.
            (LimbData::Heap(l_ptr), LimbData::Stack(_r)) => {
                // SAFETY: `len - 1` is within the bounds of `l_ptr`.
                let l = unsafe { *l_ptr.add(self.len.get() - 1) };

                // The heap value has a larger absolute value, so check its
                // sign bit.
                let l_bit = l.repr_ne() >> SHIFT;

                match l_bit {
                    // Heap value is negative.
                    1 => Ordering::Less,
                    // Heap value is positive.
                    _ => Ordering::Greater,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::cmp::Ordering;
    use core::num::NonZeroUsize;

    macro_rules! assert_cmp {
        ($l:expr, $r:expr, $ord:ident) => {{
            let result = $l.cmp(&$r);
            assert!(
                result == Ordering::$ord,
                concat!(
                    "comparison failed:\nleft: {:?}\nright: {:?}\nexpected: ",
                    stringify!($ord),
                    "\nresult: {:?}",
                ),
                $l,
                $r,
                result
            );
        }};
    }

    // TODO: Document the branches that these tests cover.

    #[test]
    fn stack_stack_pos_pos() {
        let l = ApInt::from(11212);
        let r = ApInt::from(32142);
        assert_cmp!(l, r, Less);
    }

    #[test]
    fn stack_stack_neg_neg() {
        let l = ApInt::from(-1241);
        let r = ApInt::from(-35351);
        assert_cmp!(l, r, Greater);
    }

    #[test]
    fn stack_stack_pos_neg() {
        let l = ApInt::from(21241);
        let r = ApInt::from(-35351);
        assert_cmp!(l, r, Greater);
    }

    #[test]
    fn stack_stack_neg_pos() {
        let l = ApInt::from(-15241);
        let r = ApInt::from(5351);
        assert_cmp!(l, r, Less);
    }

    #[test]
    fn stack_heap_pos() {
        let l = ApInt::from(11212);
        let r = ApInt::from(i128::MAX);
        assert_cmp!(l, r, Less);
    }

    #[test]
    fn stack_heap_neg() {
        let l = ApInt::from(11212);
        let r = ApInt::from(i128::MIN);
        assert_cmp!(l, r, Greater);
    }

    #[test]
    fn heap_stack_pos() {
        let l = ApInt::from(i128::MAX);
        let r = ApInt::from(11212);
        assert_cmp!(l, r, Greater);
    }

    #[test]
    fn heap_stack_neg() {
        let l = ApInt::from(i128::MIN);
        let r = ApInt::from(11212);
        assert_cmp!(l, r, Less);
    }

    #[test]
    fn heap_heap_pos_neg() {
        let l = ApInt::from(u128::MAX);
        let r = ApInt::from(i128::MIN);
        assert_cmp!(l, r, Greater);
    }

    #[test]
    fn heap_heap_neg_pos() {
        let l = ApInt::from(i128::MIN);
        let r = ApInt::from(i128::MAX);
        assert_cmp!(l, r, Less);
    }

    #[test]
    fn heap_heap_neg_pos_2_3() {
        #[cfg(target_pointer_width = "32")]
        let l = ApInt::from(i64::MIN);
        #[cfg(target_pointer_width = "64")]
        let l = ApInt::from(i128::MIN);

        #[cfg(target_pointer_width = "32")]
        let r = ApInt::from(u64::MAX);
        #[cfg(target_pointer_width = "64")]
        let r = ApInt::from(u128::MAX);

        assert_cmp!(l, r, Less);
    }

    // FIXME: Replace raw byte writing to set ApInt when API allows for it.

    #[test]
    fn heap_heap_neg_pos_3_2() {
        let l = unsafe {
            let mut l = ApInt::with_capacity(NonZeroUsize::new_unchecked(3));
            core::ptr::write_bytes(l.limbs_mut().as_ptr(), 0xff, 3);
            l
        };

        #[cfg(target_pointer_width = "32")]
        let r = ApInt::from(i64::MAX);
        #[cfg(target_pointer_width = "64")]
        let r = ApInt::from(i128::MAX);

        assert_cmp!(l, r, Less);
    }

    #[test]
    fn heap_heap_pos_neg_2_3() {
        #[cfg(target_pointer_width = "32")]
        let l = ApInt::from(i64::MAX);
        #[cfg(target_pointer_width = "64")]
        let l = ApInt::from(i128::MAX);

        let r = unsafe {
            let mut r = ApInt::with_capacity(NonZeroUsize::new_unchecked(3));
            core::ptr::write_bytes(r.limbs_mut().as_ptr(), 0xff, 3);
            r
        };

        assert_cmp!(l, r, Greater);
    }

    #[test]
    fn heap_heap_neg_neg_2_3() {
        #[cfg(target_pointer_width = "32")]
        let l = ApInt::from(i64::MIN);
        #[cfg(target_pointer_width = "64")]
        let l = ApInt::from(i128::MIN);

        let r = unsafe {
            let mut r = ApInt::with_capacity(NonZeroUsize::new_unchecked(3));
            core::ptr::write_bytes(r.limbs_mut().as_ptr(), 0xff, 3);
            r
        };

        assert_cmp!(l, r, Greater);
    }

    #[test]
    fn heap_heap_neg_neg_3_2() {
        let l = unsafe {
            let mut l = ApInt::with_capacity(NonZeroUsize::new_unchecked(3));
            core::ptr::write_bytes(l.limbs_mut().as_ptr(), 0xff, 3);
            l
        };

        #[cfg(target_pointer_width = "32")]
        let r = ApInt::from(i64::MIN);
        #[cfg(target_pointer_width = "64")]
        let r = ApInt::from(i128::MIN);

        assert_cmp!(l, r, Less);
    }

    #[test]
    fn heap_heap_pos_pos_2_3() {
        #[cfg(target_pointer_width = "32")]
        let l = ApInt::from(i64::MAX);
        #[cfg(target_pointer_width = "64")]
        let l = ApInt::from(i128::MAX);

        #[cfg(target_pointer_width = "32")]
        let r = ApInt::from(u64::MAX);
        #[cfg(target_pointer_width = "64")]
        let r = ApInt::from(u128::MAX);

        assert_cmp!(l, r, Less);
    }

    #[test]
    fn heap_heap_pos_pos_3_2() {
        #[cfg(target_pointer_width = "32")]
        let l = ApInt::from(u64::MAX);
        #[cfg(target_pointer_width = "64")]
        let l = ApInt::from(u128::MAX);

        #[cfg(target_pointer_width = "32")]
        let r = ApInt::from(i64::MAX);
        #[cfg(target_pointer_width = "64")]
        let r = ApInt::from(i128::MAX);

        assert_cmp!(l, r, Greater);
    }
}
