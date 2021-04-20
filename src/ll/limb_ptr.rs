use crate::int::repr::ReprLen;
use crate::ll::limb::Limb;

// Expand arguments if debug_assertions are enabled.
cfg_if::cfg_if! {
    if #[cfg(debug_assertions)] {
        macro_rules! if_debug_assertions {
            ($($arg:tt)*) => { $($arg)* };
        }
    } else {
        macro_rules! if_debug_assertions {
            ($($arg:tt)*) => {};
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(not(debug_assertions), repr(transparent))]
pub(crate) struct LimbPtr {
    ptr: *const Limb,
    #[cfg(debug_assertions)]
    bounds: Bounds,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(not(debug_assertions), repr(transparent))]
pub(crate) struct LimbMutPtr {
    ptr: *mut Limb,
    #[cfg(debug_assertions)]
    bounds: Bounds,
}

macro_rules! limb_ptr {
    ($ty:ident($ptr:ty)) => {
        impl $ty {
            #[cfg_attr(not(debug_assertions), inline(always))]
            pub fn new(ptr: $ptr, len: ReprLen) -> $ty {
                $ty {
                    ptr,
                    #[cfg(debug_assertions)]
                    bounds: Bounds::new(ptr as usize, len.len()),
                }
            }

            #[cfg_attr(not(debug_assertions), inline(always))]
            pub unsafe fn offset(self, offset: isize) -> $ty {
                if_debug_assertions!(self.bounds.validate_offset(self.ptr as usize, offset));
                $ty {
                    // SAFETY: The caller must uphold the safety requirements.
                    ptr: self.ptr.offset(offset),
                    #[cfg(debug_assertions)]
                    bounds: self.bounds,
                }
            }

            #[cfg_attr(not(debug_assertions), inline(always))]
            pub unsafe fn deref(&self) -> &Limb {
                if_debug_assertions!(self.bounds.validate_deref(self.ptr as usize));
                // SAFETY: The caller must uphold the safety requirements.
                &*self.ptr
            }
        }
    };
}

limb_ptr![LimbPtr(*const Limb)];
limb_ptr![LimbMutPtr(*mut Limb)];

impl LimbMutPtr {
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub unsafe fn deref_mut(&mut self) -> &mut Limb {
        if_debug_assertions!(self.bounds.validate_deref(self.ptr as usize));
        // SAFETY: The caller must uphold the safety requirements.
        &mut *self.ptr
    }
}

if_debug_assertions! {
    #[derive(Clone, Copy)]
    struct Bounds {
        lo: usize,
        hi: usize,
    }

    impl Bounds {
        fn new(ptr: usize, len: usize) -> Bounds {
            let bytes = len * Limb::SIZE;

            let lo = ptr;
            let hi = match lo.checked_add(bytes) {
                Some(hi) => hi,
                None => offset_overflow(ptr, bytes as isize),
            };

            Bounds { lo, hi }
        }

        fn validate_deref(self, ptr: usize) {
            // We cannot deref past the end of the block.
            if !(self.lo <= ptr && ptr < self.hi) {
                invalid_deref(ptr, self.lo, self.hi);
            }
        }

        fn validate_offset(self, ptr: usize, offset: isize) {
            let bytes = offset * Limb::SIZE as isize;

            let result = if bytes > 0 {
                ptr.checked_add(bytes.unsigned_abs())
            } else {
                ptr.checked_sub(bytes.unsigned_abs())
            };
            let offset_ptr = match result {
                Some(ptr) => ptr,
                None => offset_overflow(ptr, bytes),
            };

            // We can have a pointer offset one byte past the end of a block.
            if !(self.lo <= offset_ptr && offset_ptr <= self.hi) {
                invalid_offset(ptr, offset, self.lo, self.hi);
            }
        }
    }

    #[cold]
    #[track_caller]
    fn invalid_deref(ptr: usize, lo: usize, hi: usize) -> ! {
        panic!(
            "cannot deref pointer {:?}, must be in range {:?}..{:?}",
            PtrDebug(ptr),
            PtrDebug(lo),
            PtrDebug(hi),
        );
    }

    #[cold]
    #[track_caller]
    fn invalid_offset(ptr: usize, offset: isize, lo: usize, hi: usize) -> ! {
        panic!(
            "invalid offset {} from {:?}, must be in range {:?}..={:?}",
            offset,
            PtrDebug(ptr),
            PtrDebug(lo),
            PtrDebug(hi),
        );
    }

    #[cold]
    #[track_caller]
    fn offset_overflow(ptr: usize, offset: isize) -> ! {
        panic!("offset {} from {:?} overflows", offset, PtrDebug(ptr));
    }

    impl core::fmt::Debug for Bounds {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let mut ds = f.debug_struct("Bounds");
            ds.field("lo", &PtrDebug(self.lo));
            ds.field("lo", &PtrDebug(self.hi));
            ds.finish()
        }
    }

    struct PtrDebug(usize);

    impl core::fmt::Debug for PtrDebug {
        #[inline(always)]
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            core::fmt::Pointer::fmt(&(self.0 as *const ()), f)
        }
    }
}
