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
                let l_bit = l.repr() >> SHIFT;
                let r_bit = r.repr() >> SHIFT;

                match (l_bit, r_bit) {
                    (0, 1) => return Ordering::Greater,
                    (1, 0) => return Ordering::Less,
                    _ => {}
                }

                // Same sign bits, compare number of limbs.
                match self.len.cmp(&other.len) {
                    Ordering::Equal => {}
                    ordering => return ordering,
                }

                // At this point it is guaranteed that both ints have the same
                // number of limbs.

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
                let r_bit = r.repr() >> SHIFT;

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
                let l_bit = l.repr() >> SHIFT;

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
