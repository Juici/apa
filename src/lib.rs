//! An arbitrary-precision arithmetic library.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

mod alloc;
mod int;
mod ll;

pub use crate::int::{Int, Sign};
