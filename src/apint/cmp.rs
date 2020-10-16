use core::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};

use crate::apint::{ApInt, LimbData};

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

// TODO: Implement Ord and PartialOrd.
