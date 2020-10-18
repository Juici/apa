// Adapted from ramp limb_ptr.
// https://github.com/Aatch/ramp/blob/master/src/ll/limb_ptr.rs

use core::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use core::fmt;
use core::marker::PhantomData;
use core::num::NonZeroUsize;
use core::ops::{Deref, DerefMut};
use core::ptr::{self, NonNull};

use crate::limb::Limb;

#[derive(Clone, Copy, Debug)]
pub struct Limbs<'a> {
    ptr: NonNull<Limb>,
    bounds: Bounds,
    _marker: &'a PhantomData<()>,
}

#[derive(Clone, Copy, Debug)]
pub struct LimbsMut<'a> {
    ptr: NonNull<Limb>,
    bounds: Bounds,
    _marker: &'a PhantomData<()>,
}

macro_rules! impl_limbs {
    ($ty:ident<$lifetime:lifetime>, $ptr:ty) => {
        impl<$lifetime> $ty<$lifetime> {
            /// Creates a new limbs pointer, pointing at `ptr` valid to `ptr.add(len)`.
            ///
            /// The pointer is valid for the lifetime of the `PhantomData`.
            #[inline]
            pub unsafe fn new(
                ptr: NonNull<Limb>,
                len: NonZeroUsize,
                marker: &$lifetime PhantomData<()>,
            ) -> $ty<$lifetime> {
                debug_assert!(len.get() > 1, "invalid limbs pointer length 1");
                $ty {
                    ptr,
                    bounds: Bounds::new(ptr.as_ptr() as usize, len),
                    _marker: marker,
                }
            }

            /// Calculates the offset limbs pointer.
            ///
            /// `count` is in units of `Limb`; eg. a `count` of 3 represents a pointer
            /// offset of `3 * size_of::<Limb>()`.
            #[inline]
            pub unsafe fn add(self, count: usize) -> $ty<$lifetime> {
                debug_assert!(
                    self.bounds.is_valid_offset(self.as_ptr() as usize, count),
                    "invalid offset `{}` from `{:?}`, should be in bounds: {:?}",
                    count, self.ptr, self.bounds,
                );
                $ty {
                    // SAFETY: `ptr` is guaranteed to be non-null,
                    //         and valid for count as asserted by caller.
                    ptr: NonNull::new_unchecked(self.ptr.as_ptr().add(count)),
                    bounds: self.bounds,
                    _marker: self._marker,
                }
            }

            /// Returns the internal raw pointer.
            #[inline(always)]
            pub const fn as_ptr(self) -> $ptr {
                self.ptr.as_ptr() as $ptr
            }
        }

        impl<$lifetime> PartialEq for $ty<$lifetime> {
            #[inline]
            fn eq(&self, other: &$ty<$lifetime>) -> bool {
                self.ptr == other.ptr
            }
        }
        impl<$lifetime> Eq for $ty<$lifetime> {}

        impl<$lifetime> PartialOrd for $ty<$lifetime> {
            #[inline]
            fn partial_cmp(&self, other: &$ty<$lifetime>) -> Option<Ordering> {
                self.ptr.partial_cmp(&other.ptr)
            }
        }
        impl<$lifetime> Ord for $ty<$lifetime> {
            #[inline]
            fn cmp(&self, other: &$ty<$lifetime>) -> Ordering {
                self.ptr.cmp(&other.ptr)
            }
        }

        impl<$lifetime> Deref for $ty<$lifetime> {
            type Target = Limb;

            #[inline]
            fn deref(&self) -> &Limb {
                debug_assert!(
                    self.bounds.can_deref(self.as_ptr() as usize),
                    "invalid deref of `{:?}`, should be in bounds: {:?}",
                    self.ptr, self.bounds,
                );
                // SAFETY: `ptr` is guaranteed to be non-null.
                unsafe { self.ptr.as_ref() }
            }
        }
    };
}

impl_limbs!(Limbs<'a>, *const Limb);
impl_limbs!(LimbsMut<'a>, *mut Limb);

impl<'a> DerefMut for LimbsMut<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Limb {
        // SAFETY: `ptr` is guaranteed to be non-null.
        unsafe { self.ptr.as_mut() }
    }
}

impl<'a> LimbsMut<'a> {
    /// Returns a constant view of limbs.
    ///
    /// Equivalent to a cast from `*mut Limb` to `*const Limb`.
    #[inline]
    pub const fn as_const(self) -> Limbs<'a> {
        Limbs {
            ptr: self.ptr,
            bounds: self.bounds,
            _marker: self._marker,
        }
    }

    /// Copy `count` limbs from `src` to `self`.
    ///
    /// `src` and `self` must *not* overlap.
    #[inline]
    pub unsafe fn copy_nonoverlapping(&mut self, src: Limbs, count: NonZeroUsize) {
        // Check source and destination can be dereferenced for the whole range
        // of count.
        debug_assert!(self.bounds.can_deref(self.as_ptr() as usize));
        debug_assert!(
            self.bounds
                .is_valid_offset(self.as_ptr() as usize, count.get())
        );
        debug_assert!(src.bounds.can_deref(src.as_ptr() as usize));
        debug_assert!(
            src.bounds
                .is_valid_offset(src.as_ptr() as usize, count.get())
        );

        // Check bounds don't overlap.
        debug_assert!(
            !self.bounds.is_overlapping(src.bounds),
            "limbs overlap:\ndst: {:?}\nsrc: {:?}",
            self,
            src
        );

        ptr::copy_nonoverlapping(src.as_ptr(), self.as_ptr(), count.get());
    }
}

#[cfg(debug_assertions)]
#[derive(Clone, Copy)]
struct Bounds {
    lo: usize,
    hi: usize,
}

#[cfg(not(debug_assertions))]
#[derive(Clone, Copy)]
struct Bounds;

// Bounds checks for sanity in debug builds.

#[cfg(debug_assertions)]
impl Bounds {
    const fn new(ptr: usize, len: NonZeroUsize) -> Bounds {
        Bounds {
            lo: ptr,
            hi: ptr + (len.get() * Limb::SIZE),
        }
    }

    const fn can_deref(self, ptr: usize) -> bool {
        // Cannot deref at the limit.
        self.lo <= ptr && ptr < self.hi
    }

    const fn is_valid_offset(self, ptr: usize, count: usize) -> bool {
        let bytes = count * Limb::SIZE;
        // When using `add` a pointer cannot rely on wrapping.
        match ptr.checked_add(bytes) {
            // An offset is still valid at the limit, but cannot deref.
            Some(offset_ptr) => self.lo <= offset_ptr && offset_ptr <= self.hi,
            None => false,
        }
    }

    const fn is_overlapping(self, other: Bounds) -> bool {
        (self.lo > other.lo && self.lo < other.hi) || (self.hi > other.lo && self.hi < other.hi)
    }
}

// Optimise out bounds checks in release builds.

#[cfg(not(debug_assertions))]
impl Bounds {
    #[inline(always)]
    const fn new(_ptr: usize, _len: NonZeroUsize) -> Bounds {
        Bounds
    }

    #[inline(always)]
    const fn can_deref(self, _ptr: usize) -> bool {
        true
    }

    #[inline(always)]
    const fn is_valid_offset(self, _ptr: usize, _offset: usize) -> bool {
        true
    }

    #[inline(always)]
    const fn is_overlapping(self, _other: Bounds) -> bool {
        false
    }
}

impl fmt::Debug for Bounds {
    #[cfg(debug_assertions)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut bounds = f.debug_struct("Bounds");
        bounds.field("lo", &format_args!("{:#x}", self.lo));
        bounds.field("hi", &format_args!("{:#x}", self.hi));
        bounds.finish()
    }

    #[cfg(not(debug_assertions))]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bounds {{ <optimized out> }}")
    }
}
