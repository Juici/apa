use core::alloc::Layout;
use core::mem;
use core::num::NonZeroUsize;
use core::ptr::{self, NonNull};

use crate::alloc;
use crate::ll::limb::Limb;
use crate::ll::limb_ptr::{LimbMutPtr, LimbPtr};

use super::{Int, Sign};

/// Internal storage for `Int` using one machine word.
pub union Repr {
    pub inline: Limb,
    pub ptr: NonNull<Limb>,
}

static_assertions::assert_eq_size!(Repr, Limb);

/// The number of limbs in the internal representation of an `Int`.
///
/// The length is represented as a signed integer, with the sign indicating the
/// sign of the integer.
///
/// # Sign
///
/// - `len == 0` means the integer is zero.
/// - `len < 0` means the integer is negative.
/// - `len > 0` means the integer is positive.
///
/// # Representation
///
/// - `len.abs() <= 1` means the [`Repr`] is inline.
/// - `len.abs() > 1` means the [`Repr`] uses a heap allocation.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ReprLen(i32);

static_assertions::assert_eq_size!(ReprLen, i32);

impl ReprLen {
    /// Returns a [`ReprLen`] with the given value.
    #[inline(always)]
    pub const fn new(len: i32) -> ReprLen {
        ReprLen(len)
    }

    /// Returns the internal representation of the length.
    #[inline(always)]
    pub const fn repr(self) -> i32 {
        self.0
    }

    /// Returns the magnitude of `self`.
    #[inline(always)]
    pub const fn len(self) -> usize {
        self.0.unsigned_abs() as usize
    }

    /// Returns if [`Repr`] is inline.
    #[inline(always)]
    pub const fn is_inline(self) -> bool {
        matches!(self.0, -1 | 0 | 1)
    }

    /// Returns the [`Sign`] of the [`Int`].
    #[inline(always)]
    pub const fn sign(self) -> Sign {
        match self.0 {
            n if n > 0 => Sign::Positive,
            0 => Sign::Zero,
            _ => Sign::Negative,
        }
    }

    /// Returns a [`ReprLen`] representing the magnitude of `self`.
    #[inline(always)]
    pub const fn abs(self) -> ReprLen {
        // We should never encounter a scenario where we overflow,
        // since we would probably have run out of memory already.
        ReprLen(self.0.abs())
    }
}

impl Int {
    /// Returns a pointer to the first limb in `self`.
    #[inline(always)]
    pub(crate) fn as_ptr(&self) -> LimbPtr {
        let ptr = if self.len.is_inline() {
            // SAFETY: Representation is inline.
            unsafe { &self.repr.inline as *const Limb }
        } else {
            // SAFETY: Representation is heap allocated.
            unsafe { self.repr.ptr.as_ptr() }
        };
        LimbPtr::new(ptr, self.len)
    }

    /// Returns a mutable pointer to the first limb in `self`.
    #[inline(always)]
    pub(crate) fn as_mut_ptr(&mut self) -> LimbMutPtr {
        let ptr = if self.len.is_inline() {
            // SAFETY: Representation is inline.
            unsafe { &mut self.repr.inline as *mut Limb }
        } else {
            // SAFETY: Representation is heap allocated.
            unsafe { self.repr.ptr.as_ptr() }
        };
        LimbMutPtr::new(ptr, self.len)
    }

    /// Returns an [`Int`] with a single unsigned limb.
    #[inline]
    pub(crate) const fn from_limb(limb: Limb) -> Int {
        let repr = Repr { inline: limb };
        let len = match limb.repr() {
            0 => ReprLen(0),
            _ => ReprLen(1),
        };
        Int { repr, len }
    }

    /// Allocates an [`Int`] with `len` limbs.
    ///
    /// # Safety
    ///
    /// The caller must guarantee `len < -1 || len > 1`.
    unsafe fn allocate(len: i32) -> Int {
        let len = ReprLen(len);

        debug_assert!(!len.is_inline());

        let layout = match Layout::array::<Limb>(len.len()) {
            Ok(layout) => layout,
            Err(_) => capacity_overflow(),
        };
        match alloc_guard(layout.size()) {
            Ok(_) => {}
            Err(_) => capacity_overflow(),
        }
        // SAFETY: `layout.size() > 0` is guaranteed, since the caller
        //         guarantees `len.len() > 1` and `Limb` is not a ZST.
        let ptr = alloc::allocate_zeroed(layout);

        let repr = Repr { ptr: ptr.cast() };
        Int { repr, len }
    }

    /// Returns `None` if [`Repr`] is inline, otherwise returns a pointer to the
    /// allocation and the memory layout.
    fn current_allocation(&self) -> Option<(NonNull<u8>, Layout)> {
        if self.len.is_inline() {
            None
        } else {
            static_assertions::const_assert!(Limb::SIZE != 0);
            static_assertions::const_assert!(Limb::ALIGN != 0);
            static_assertions::const_assert!(Limb::ALIGN.is_power_of_two());

            let size = Limb::SIZE * self.len.len();
            let align = Limb::ALIGN;

            // SAFETY: Our representation is heap allocated.
            let ptr = unsafe { self.repr.ptr.cast() };

            // SAFETY: We already have an allocated block of memory, so we can
            //         bypass runtime checks to get our current layout.
            let layout = unsafe { Layout::from_size_align_unchecked(size, align) };

            Some((ptr, layout))
        }
    }
}

impl Clone for Int {
    fn clone(&self) -> Self {
        let repr = match self.current_allocation() {
            None => Repr {
                // SAFETY: Our representation is inline.
                inline: unsafe { self.repr.inline },
            },
            Some((src, layout)) => {
                // Don't bother allocating zeroed memory, since we will
                // overwrite it in the `ptr::copy_nonoverlapping` call.

                // SAFETY: We already have an allocated block of memory, so we can
                //         bypass runtime checks on the size of layout.
                let dst = unsafe { alloc::allocate(layout) };

                // SAFETY: `src` is valid for reads of `layout.size()` bytes.
                //         `dst` is valid for writes of `layout.size()` bytes.
                //         `src` and `dst` are nonoverlapping.
                unsafe { ptr::copy_nonoverlapping(src.as_ptr(), dst.as_ptr(), layout.size()) };

                Repr { ptr: dst.cast() }
            }
        };
        Int {
            repr,
            len: self.len,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        match source.current_allocation() {
            None => {
                // We drop `self`, in favour of creating a clone of `source`.
                // This allows us to reuse our existing `Drop` and `Clone::clone`
                // implementations.
                *self = source.clone();
            }
            Some((src, new_layout)) => {
                let dst = match self.current_allocation() {
                    // SAFETY: We already have an allocated block of memory, so
                    //         we can bypass runtime checks on the size of layout.
                    None => unsafe { alloc::allocate(new_layout) },

                    Some((mut dst, old_layout)) => {
                        // If the layouts differ in size, we will attempt to
                        // resize the allocation referenced by `dst`.
                        if old_layout.size() != new_layout.size() {
                            static_assertions::const_assert!(Limb::SIZE != 0);

                            let new_size = new_layout.size();
                            // SAFETY: `new_size > 0` is guaranteed, since `Limb` is not a ZST
                            //         and source has more than 1 limb.
                            let new_size = unsafe { NonZeroUsize::new_unchecked(new_size) };

                            // SAFETY: We already have an allocated block of memory, so we can
                            //         bypass runtime checks on new_size overflowing.
                            dst = unsafe { alloc::reallocate(dst, old_layout, new_size) };
                        }

                        // `dst` is guaranteed to have the same layout as `src` now.
                        dst
                    }
                };

                // SAFETY: `src` is valid for reads of `new_layout.size()` bytes.
                //         `dst` is valid for writes of `new_layout.size()` bytes.
                //         `src` and `dst` are nonoverlapping.
                unsafe { ptr::copy_nonoverlapping(src.as_ptr(), dst.as_ptr(), new_layout.size()) };

                // Update `self` length to match `source` length.
                self.len = source.len;
            }
        }
    }
}

impl Drop for Int {
    fn drop(&mut self) {
        // There is no need to drop the limbs, so we just deallocate if our
        // representation is heap allocated.
        static_assertions::const_assert!(!mem::needs_drop::<Limb>());

        if let Some((ptr, layout)) = self.current_allocation() {
            // SAFETY: `ptr` points to our heap allocation, and
            //         `layout` fits the allocation.
            unsafe { alloc::deallocate(ptr, layout) };
        }
    }
}

// `Int` can safely be sent across thread boundaries, since it does not own
// aliasing memory and has no reference counting mechanism.
unsafe impl Send for Int {}
// `Int` can safely be shared between threads, since it does not own
// aliasing memory and has no mutable internal state.
unsafe impl Sync for Int {}

// We need to guarantee the following:
// - We don't ever allocate `> isize::MAX` byte-size objects.
// - We don't overflow `usize::MAX` and actually allocate too little.
//
// On 64-bit we just need to check for overflow since trying to allocate
// `> isize::MAX` bytes will surely fail. On 32-bit and 16-bit we need to add
// an extra guard for this in case we're running on a platform which can use
// all 4GB in user-space, e.g., PAE or x32.

struct CapacityOverflow;

#[inline]
fn alloc_guard(alloc_size: usize) -> Result<(), CapacityOverflow> {
    // HACK: This exists because `usize::BITS` is currently gated behind the
    //       `int_bits_const` feature.
    const USIZE_BITS: usize = mem::size_of::<usize>() * 8;

    if USIZE_BITS < 64 && alloc_size > isize::MAX as usize {
        Err(CapacityOverflow)
    } else {
        Ok(())
    }
}

// One central function responsible for reporting capacity overflows. This will
// ensure that the code generation related to these panics is minimal as there
// is only one location which panics rather than a bunch throughout the module.
#[cold]
#[track_caller]
fn capacity_overflow() -> ! {
    panic!("capacity overflow");
}
