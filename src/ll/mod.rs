use crate::ll::limb::Limb;
use crate::ll::limb_ptr::LimbPtr;

pub mod limb;
pub mod limb_ptr;

/// Compare the limbs of two integers for equality.
///
/// In the case that `len == 0`, both integers are considered to be zero and
/// the comparison automatically succeeds without comparing limbs.
#[inline]
pub unsafe fn eq(lp: LimbPtr, rp: LimbPtr, len: usize) -> bool {
    let n = len * Limb::SIZE;
    // SAFETY: `lp` and `rp` are valid for reads of `len * size_of::<Limb>()` bytes.
    libc::memcmp(lp.raw() as *const _, rp.raw() as *const _, n) == 0
}
