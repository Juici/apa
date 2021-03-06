cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        pub use std::alloc::{alloc, alloc_zeroed, dealloc, handle_alloc_error, realloc};

        pub use std::vec::Vec;
    } else {
        extern crate alloc;

        pub use alloc::alloc::{alloc, alloc_zeroed, dealloc, handle_alloc_error, realloc};

        pub use alloc::vec::Vec;
    }
}
