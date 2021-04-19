use core::marker::PhantomData;

use crate::ll::limb::Limb;

pub(crate) struct Limbs<'a> {
    ptr: *const Limb,
    _lifetime: PhantomData<&'a Limb>,
}

pub(crate) struct LimbsMut<'a> {
    ptr: *mut Limb,
    _lifetime: PhantomData<&'a Limb>,
}
