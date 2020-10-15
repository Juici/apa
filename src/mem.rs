use core::alloc::Layout;
use core::num::NonZeroUsize;
use core::ptr::NonNull;

use crate::alloc;
use crate::limb::Limb;

// TODO: Replace with allocator_api when stabilised.

// FIXME: These alloc/dealloc/realloc functions should really be `unsafe`.

pub fn alloc_limbs(capacity: NonZeroUsize) -> NonNull<Limb> {
    let layout = match Layout::array::<Limb>(capacity.get()) {
        Ok(layout) => layout,
        Err(_) => capacity_overflow(),
    };
    alloc_guard(layout.size());

    // SAFETY: This is safe since we have verified the integrity of the layout.
    let ptr = unsafe { alloc::alloc_zeroed(layout) };
    if ptr.is_null() {
        alloc::handle_alloc_error(layout);
    }

    // SAFETY: `ptr` is guaranteed to be non-null at this point.
    unsafe { NonNull::new_unchecked(ptr.cast()) }
}

pub fn dealloc_limbs(ptr: NonNull<Limb>, size: NonZeroUsize) {
    const ALIGN: usize = core::mem::align_of::<Limb>();
    const SIZE: usize = core::mem::size_of::<Limb>();

    let size = SIZE * size.get();

    // SAFETY: `ptr` is already already allocated so we can bypass checks.
    let layout = unsafe { Layout::from_size_align_unchecked(size, ALIGN) };
    // SAFETY: ptr is guaranteed to be non-null and layout is correct.
    unsafe { alloc::dealloc(ptr.cast().as_ptr(), layout) };
}

pub fn realloc_limbs(
    ptr: NonNull<Limb>,
    old_size: NonZeroUsize,
    new_size: NonZeroUsize,
) -> NonNull<Limb> {
    const ALIGN: usize = core::mem::align_of::<Limb>();
    const SIZE: usize = core::mem::size_of::<Limb>();

    let old_size = SIZE * old_size.get();
    let new_size = SIZE * new_size.get();
    alloc_guard(new_size);

    // SAFETY: `ptr` is already already allocated so we can bypass checks.
    let layout = unsafe { Layout::from_size_align_unchecked(old_size, ALIGN) };

    // SAFETY: This is safe since we have verified the integrity of the layout.
    let ptr = unsafe { alloc::realloc(ptr.cast().as_ptr(), layout, new_size) };
    if ptr.is_null() {
        alloc::handle_alloc_error(layout);
    }

    // SAFETY: ptr is guaranteed to be non-null at this point.
    unsafe { NonNull::new_unchecked(ptr.cast()) }
}

// We need to guarantee the following:
// * We don't ever allocate `> isize::MAX` byte-size objects.
// * We don't overflow `usize::MAX` and actually allocate too little.
//
// On 64-bit we just need to check for overflow since trying to allocate
// `> isize::MAX` bytes will surely fail. On 32-bit and 16-bit we need to add
// an extra guard for this in case we're running on a platform which can use
// all 4GB in user-space, e.g., PAE or x32.
//
// Taken from alloc::raw_vec module.
#[inline]
fn alloc_guard(alloc_size: usize) {
    if core::mem::size_of::<usize>() < 64 && alloc_size > isize::MAX as usize {
        capacity_overflow()
    }
}

// One central function responsible for reporting capacity overflows. This will
// ensure that the code generation related to these panics is minimal as
// there's only one location which panics rather than a bunch throughout the
// module.
//
// Taken from alloc::raw_vec module.
fn capacity_overflow() -> ! {
    panic!("capacity overflow");
}
