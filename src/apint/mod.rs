use core::fmt;
use core::marker::PhantomData;
use core::num::NonZeroUsize;
use core::ptr::NonNull;

use crate::limb::Limb;
use crate::limbs::{Limbs, LimbsMut};
use crate::mem;

mod cmp;
mod convert;
mod num;
mod ops;
mod radix;

// SAFETY: This is safe since `1` is non-zero.
const NZUSIZE_ONE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1) };

/// An arbitrary-precision integer.
pub struct ApInt {
    /// The number of limbs used to store data.
    len: NonZeroUsize,
    /// The data holding the bits of the integer.
    data: ApIntData,
}

/// A single stack allocated limb or pointer to heap allocated limbs.
union ApIntData {
    /// Inlined storage for values able to be stored within a single machine word.
    value: Limb,
    /// Heap allocated storage for values unable to be stored within a single machine word.
    ptr: NonNull<Limb>,
}

// `ApInt` can safely be sent across thread boundaries, since it does not own
// aliasing memory and has no reference counting mechanism.
unsafe impl Send for ApInt {}
// `ApInt` can safely be shared between threads, since it does not own
// aliasing memory and has no mutable internal state.
unsafe impl Sync for ApInt {}

impl ApInt {
    /// Represents an `ApInt` with value `0`.
    pub const ZERO: ApInt = ApInt::from_limb(Limb::ZERO);
    /// Represents an `ApInt` with value `1`.
    pub const ONE: ApInt = ApInt::from_limb(Limb::ONE);

    /// Creates an `ApInt` with a single limb.
    const fn from_limb(value: Limb) -> ApInt {
        ApInt {
            len: NZUSIZE_ONE,
            data: ApIntData { value },
        }
    }

    /// Creates an `ApInt` with space allocated for the given capacity.
    ///
    /// Data is zeroed.
    ///
    /// # Safety
    ///
    /// Calling this function with a capacity of `1` will result in undefined
    /// behaviour.
    fn with_capacity(capacity: NonZeroUsize) -> ApInt {
        // Sanity check when testing. Since this is an internal function we
        // should be able to guarantee it is never called with a capacity of 1.
        debug_assert!(
            capacity.get() > 1,
            "allocating `ApInt` with capacity 1 is not supported"
        );

        // SAFETY: This is safe since we will track this allocation.
        let ptr = unsafe { mem::alloc_limbs(capacity) };
        ApInt {
            len: capacity,
            data: ApIntData { ptr },
        }
    }
}

impl Drop for ApInt {
    fn drop(&mut self) {
        match self.len {
            NZUSIZE_ONE => {}
            // SAFETY: `ptr` is a valid pointer, since `len > 1`.
            len => unsafe { mem::dealloc_limbs(self.data.ptr, len) },
        }
    }
}

impl Clone for ApInt {
    fn clone(&self) -> Self {
        match self.data() {
            LimbData::Stack(value) => ApInt::from_limb(value),
            LimbData::Heap(src, len) => {
                let mut n = ApInt::with_capacity(len);

                // SAFETY: This is safe since `n` and `self` have the same
                //         number of limbs and do not overlap.
                unsafe { n.limbs_mut().copy_nonoverlapping(src, len) };

                n
            }
        }
    }

    fn clone_from(&mut self, source: &Self) {
        match (self.len, source.len) {
            // Both stack allocated.
            (NZUSIZE_ONE, NZUSIZE_ONE) => {
                // SAFETY: This is safe since both ints are stack allocated.
                self.data.value = unsafe { source.data.value };
            }
            // Self heap allocated, source stack allocated.
            (dst_len, NZUSIZE_ONE) => {
                // SAFETY: This is safe since self is heap allocated and has length `dst_len`.
                unsafe { mem::dealloc_limbs(self.data.ptr, dst_len) };

                // SAFETY: This is safe since source is stack allocated.
                self.data.value = unsafe { source.data.value };
                self.len = NZUSIZE_ONE;
            }
            // Self stack allocated, source heap allocated.
            (NZUSIZE_ONE, src_len) => {
                // SAFETY: This is safe since we will track this allocation.
                let dst = unsafe { mem::alloc_limbs(src_len) };

                self.data.ptr = dst;
                self.len = src_len;

                // SAFETY: This is safe since `self` and `source` have the same
                //         number of limbs and do not overlap.
                unsafe {
                    self.limbs_mut()
                        .copy_nonoverlapping(source.limbs(), src_len);
                }
            }
            // Both heap allocated.
            (old_len, src_len) => {
                // Reallocate destination if lengths differ.
                if old_len != src_len {
                    // SAFETY: This is safe since self is heap allocated and has length `old_len`.
                    unsafe { self.data.ptr = mem::realloc_limbs(self.data.ptr, old_len, src_len) };
                }

                // SAFETY: This is safe since `self` and `source` have the same
                //         number of limbs and do not overlap.
                unsafe {
                    self.limbs_mut()
                        .copy_nonoverlapping(source.limbs(), src_len);
                }
            }
        }
    }
}

impl fmt::Debug for ApInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut int = f.debug_struct("ApInt");

        int.field("len", &self.len);

        // TODO: Improve debug implementation.
        match self.data() {
            LimbData::Stack(value) => int.field("limbs", &[value]),
            // SAFETY: `limbs` are for reads up to `len`.
            LimbData::Heap(limbs, len) => int.field("limbs", unsafe {
                &core::slice::from_raw_parts(limbs.as_ptr(), len.get())
            }),
        };

        int.finish()
    }
}

pub(crate) enum LimbData<'a> {
    Stack(Limb),
    Heap(Limbs<'a>, NonZeroUsize),
}

pub(crate) enum LimbDataMut<'a> {
    Stack(&'a mut Limb),
    Heap(LimbsMut<'a>, NonZeroUsize),
}

impl ApInt {
    /// Returns an accessor to the limb data.
    #[inline]
    pub(crate) fn data(&self) -> LimbData {
        match self.len {
            // SAFETY: A len of 1 guarantees that value is a valid limb.
            NZUSIZE_ONE => LimbData::Stack(unsafe { self.data.value }),
            // SAFETY: A len greater than 1 guarantees that ptr is a valid pointer.
            len => LimbData::Heap(unsafe { self.limbs() }, len),
        }
    }

    /// Returns a mutable accessor to the limb data.
    #[inline]
    pub(crate) fn data_mut(&mut self) -> LimbDataMut {
        match self.len {
            // SAFETY: A len of 1 guarantees that value is a valid limb.
            NZUSIZE_ONE => LimbDataMut::Stack(unsafe { &mut self.data.value }),
            // SAFETY: A len greater than 1 guarantees that ptr is a valid pointer.
            len => LimbDataMut::Heap(unsafe { self.limbs_mut() }, len),
        }
    }

    /// Returns a pointer accessor to the limb data.
    ///
    /// This function doesn't check that the internal data representation is a
    /// valid pointer.
    #[inline]
    pub(crate) unsafe fn limbs(&self) -> Limbs {
        Limbs::new(self.data.ptr, self.len, &PhantomData)
    }

    /// Returns a mutable pointer accessor to the limb data.
    ///
    /// This function doesn't check that the internal data representation is a
    /// valid pointer.
    #[inline]
    pub(crate) unsafe fn limbs_mut(&mut self) -> LimbsMut {
        LimbsMut::new(self.data.ptr, self.len, &PhantomData)
    }
}
