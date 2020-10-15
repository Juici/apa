use core::hint;
use core::num::NonZeroUsize;
use core::ptr::{self, NonNull};

use crate::limb::Limb;
use crate::mem;

mod convert;

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

        let ptr = mem::alloc_limbs(capacity);
        ApInt {
            len: capacity,
            data: ApIntData { ptr },
        }
    }
}

impl Drop for ApInt {
    fn drop(&mut self) {
        if self.len.get() > 1 {
            // SAFETY: `ptr` is a valid pointer, since `len > 1`.
            mem::dealloc_limbs(unsafe { self.data.ptr }, self.len);
        }
    }
}

impl Clone for ApInt {
    fn clone(&self) -> Self {
        match self.storage() {
            ApIntStorage::Stack(value) => ApInt::from_limb(value),
            ApIntStorage::Heap(src) => {
                let n = ApInt::with_capacity(self.len);

                // SAFETY: This safe since `n` is heap allocated.
                let dst = unsafe { n.data.ptr };
                // SAFETY: `src` and `dst` are valid pointers for `len` limbs.
                unsafe { ptr::copy_nonoverlapping(src.as_ptr(), dst.as_ptr(), self.len.get()) };

                n
            }
        }
    }

    fn clone_from(&mut self, source: &Self) {
        match (self.len.get(), source.len.get()) {
            // SAFETY: `len` is non-zero.
            (0, _) | (_, 0) => unsafe { hint::unreachable_unchecked() },
            // Both stack allocated.
            (1, 1) => {
                // SAFETY: This is safe since both ints are stack allocated.
                self.data.value = unsafe { source.data.value };
            }
            // Self heap allocated, source stack allocated.
            (_, 1) => {
                {
                    // SAFETY: This is safe since self is heap allocated.
                    let dst = unsafe { self.data.ptr };
                    mem::dealloc_limbs(dst, self.len);
                }

                // SAFETY: This is safe since source is stack allocated.
                self.data.value = unsafe { source.data.value };
                self.len = NZUSIZE_ONE;
            }
            // Self stack allocated, source heap allocated.
            (1, src_len) => {
                let dst = mem::alloc_limbs(source.len);

                // SAFETY: This safe since the source is heap allocated.
                let src = unsafe { source.data.ptr };
                // SAFETY: This is safe since both ints have the same length.
                unsafe { ptr::copy_nonoverlapping(src.as_ptr(), dst.as_ptr(), src_len) };

                self.data.ptr = dst;
                self.len = source.len;
            }
            // Both heap allocated.
            (dst_len, src_len) => {
                // SAFETY: This is safe since self is heap allocated.
                let mut dst = unsafe { self.data.ptr };
                if src_len != dst_len {
                    dst = mem::realloc_limbs(dst, self.len, source.len)
                }

                // SAFETY: This safe since the source is heap allocated.
                let src = unsafe { source.data.ptr };
                // SAFETY: This is safe since both ints have the same length.
                unsafe { ptr::copy_nonoverlapping(src.as_ptr(), dst.as_ptr(), src_len) };
            }
        }
    }
}

enum ApIntStorage<'a> {
    Stack(Limb),
    Heap(&'a NonNull<Limb>),
}

enum ApIntStorageMut<'a> {
    Stack(&'a mut Limb),
    Heap(&'a mut NonNull<Limb>),
}

impl ApInt {
    // TODO: Replace with a proper API.

    /// Returns a storage accessor for the limb data.
    #[inline]
    fn storage(&self) -> ApIntStorage {
        match self.len.get() {
            // SAFETY: The len is non-zero.
            0 => unsafe { hint::unreachable_unchecked() },
            // SAFETY: A len of 1 guarantees that value is a valid limb.
            1 => ApIntStorage::Stack(unsafe { self.data.value }),
            // SAFETY: A len greater than 1 guarantees that ptr is a valid pointer.
            _ => ApIntStorage::Heap(unsafe { &self.data.ptr }),
        }
    }

    /// Returns a mutable storage accessor for the limb data.
    #[inline]
    fn storage_mut(&mut self) -> ApIntStorageMut {
        match self.len.get() {
            // SAFETY: The len is non-zero.
            0 => unsafe { hint::unreachable_unchecked() },
            // SAFETY: A len of 1 guarantees that value is a valid limb.
            1 => ApIntStorageMut::Stack(unsafe { &mut self.data.value }),
            // SAFETY: A len greater than 1 guarantees that ptr is a valid pointer.
            _ => ApIntStorageMut::Heap(unsafe { &mut self.data.ptr }),
        }
    }

    // TODO: Add proper limb accessor/iterator.

    /// Returns the limb at the given index.
    pub(crate) unsafe fn limb(&self, index: usize) -> Limb {
        match self.storage() {
            ApIntStorage::Stack(limb) => limb,
            ApIntStorage::Heap(ptr) => *ptr.as_ptr().add(index),
        }
    }

    /// Returns a mutable reference to the limb at the given index.
    pub(crate) unsafe fn limb_mut(&mut self, index: usize) -> &mut Limb {
        match self.storage_mut() {
            ApIntStorageMut::Stack(limb) => limb,
            ApIntStorageMut::Heap(ptr) => &mut *ptr.as_ptr().add(index),
        }
    }
}
