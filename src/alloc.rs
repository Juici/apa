// TODO: Use allocator_api once stabilised.

#![allow(dead_code)]

mod alloc {
    cfg_if::cfg_if! {
        if #[cfg(feature = "std")] {
            pub use std::alloc::*;
        } else {
            extern crate alloc as alloc_crate;

            pub use alloc_crate::alloc::*;
        }
    }
}

use core::alloc::Layout;
use core::num::NonZeroUsize;
use core::ptr::NonNull;

/// Allocates a block of memory.
///
/// # Safety
///
/// The caller must guarantee `layout.size() > 0`.
#[inline]
pub unsafe fn allocate(layout: Layout) -> NonNull<u8> {
    // SAFETY: `layout.size() > 0` must be guaranteed by caller.
    match NonNull::new(alloc::alloc(layout)) {
        Some(ptr) => ptr,
        None => alloc::handle_alloc_error(layout),
    }
}

/// Allocates a block of zero-initialised memory.
///
/// # Safety
///
/// The caller must guarantee `layout.size() > 0`.
#[inline]
pub unsafe fn allocate_zeroed(layout: Layout) -> NonNull<u8> {
    // SAFETY: `layout.size() > 0` must be guaranteed by caller.
    match NonNull::new(alloc::alloc_zeroed(layout)) {
        Some(ptr) => ptr,
        None => alloc::handle_alloc_error(layout),
    }
}

/// Deallocates the memory referenced by `ptr`.
///
/// # Safety
///
/// - `ptr` must point to a block of memory currently allocated by this module.
/// - `layout` must fit the the block of memory referenced by `ptr`.
#[inline]
pub unsafe fn deallocate(ptr: NonNull<u8>, layout: Layout) {
    // SAFETY: `ptr` is guaranteed to be non-null,
    //         and other constraints are guaranteed by caller.
    alloc::dealloc(ptr.as_ptr(), layout);
}

/// Resizes the block of memory referenced by `ptr`, with layout `layout`,
/// to the the given `new_size`.
///
/// # Safety
///
/// - `ptr` must point to a block of memory currently allocated by this module.
/// - `layout` must fit the the block of memory referenced by `ptr`.
/// - `new_size` when rounded up to the nearest multiple of `layout.align()`
///   must not overflow (ie. must be less than `usize::MAX`).
#[inline]
pub unsafe fn reallocate(ptr: NonNull<u8>, layout: Layout, new_size: NonZeroUsize) -> NonNull<u8> {
    // SAFETY: `ptr` is guaranteed to be non-null,
    //         `new_size > 0` is guaranteed,
    //         and other constraints are guaranteed by caller.
    match NonNull::new(alloc::realloc(ptr.as_ptr(), layout, new_size.get())) {
        Some(ptr) => ptr,
        // SAFETY: `layout.align()` is guaranteed to be non-zero and a power of two,
        //         and other constraints are guaranteed by the caller.
        None => alloc::handle_alloc_error(Layout::from_size_align_unchecked(
            new_size.get(),
            layout.align(),
        )),
    }
}
