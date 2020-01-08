// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! FFI tools.
//!
//! We implement all primitive types that have been needed in SAFE projects so far. More can be
//! implemented if needed, with the following exceptions, which should not be implemented:
//!
//! + `bool`: This doesn't seem to be safe to pass over the FFI directly. Should be converted to a
//! type such as `u32` instead.
//! + `char`: It's not clear why this would be necessary. You'd probably want to convert to `u32`
//! for better ABI stability.
//! + `i128` and `u128`: do not have a stable ABI, so they cannot be returned across the FFI.

/// Trait to convert between FFI and Rust representations of types.
pub trait ReprC {
    /// C representation of the type.
    type C;
    /// Error type.
    type Error;

    /// Convert from a raw FFI type into a native Rust type by cloning the data.
    ///
    /// # Safety
    ///
    /// The implementation of this function may be unsafe, as `repr_c` may be a raw pointer that
    /// needs to be valid.
    unsafe fn clone_from_repr_c(repr_c: Self::C) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl ReprC for i32 {
    type C = i32;
    type Error = ();

    unsafe fn clone_from_repr_c(repr_c: Self::C) -> Result<Self, Self::Error> {
        Ok(repr_c)
    }
}

impl ReprC for i64 {
    type C = i64;
    type Error = ();

    unsafe fn clone_from_repr_c(repr_c: Self::C) -> Result<Self, Self::Error> {
        Ok(repr_c)
    }
}

impl ReprC for u32 {
    type C = u32;
    type Error = ();

    unsafe fn clone_from_repr_c(repr_c: Self::C) -> Result<Self, Self::Error> {
        Ok(repr_c)
    }
}

impl ReprC for u64 {
    type C = u64;
    type Error = ();

    unsafe fn clone_from_repr_c(repr_c: Self::C) -> Result<Self, Self::Error> {
        Ok(repr_c)
    }
}

impl ReprC for usize {
    type C = usize;
    type Error = ();

    unsafe fn clone_from_repr_c(repr_c: Self::C) -> Result<Self, Self::Error> {
        Ok(repr_c)
    }
}

impl<T> ReprC for *const T {
    type C = *const T;
    type Error = ();

    unsafe fn clone_from_repr_c(repr_c: Self::C) -> Result<Self, Self::Error> {
        Ok(repr_c)
    }
}

impl<T> ReprC for *mut T {
    type C = *mut T;
    type Error = ();

    unsafe fn clone_from_repr_c(repr_c: Self::C) -> Result<Self, Self::Error> {
        Ok(repr_c)
    }
}

// TODO: Replace these with a const generic implementation once it is stable.
// https://github.com/rust-lang/rust/issues/44580

impl ReprC for [u8; 24] {
    type C = *const [u8; 24];
    type Error = ();

    unsafe fn clone_from_repr_c(repr_c: Self::C) -> Result<Self, Self::Error> {
        Ok(*repr_c)
    }
}

impl ReprC for [u8; 32] {
    type C = *const [u8; 32];
    type Error = ();

    unsafe fn clone_from_repr_c(repr_c: Self::C) -> Result<Self, Self::Error> {
        Ok(*repr_c)
    }
}

impl ReprC for [u8; 48] {
    type C = *const [u8; 48];
    type Error = ();

    unsafe fn clone_from_repr_c(repr_c: Self::C) -> Result<Self, Self::Error> {
        Ok(*repr_c)
    }
}

impl ReprC for [u8; 64] {
    type C = *const [u8; 64];
    type Error = ();

    unsafe fn clone_from_repr_c(repr_c: Self::C) -> Result<Self, Self::Error> {
        Ok(*repr_c)
    }
}

impl ReprC for [u8; 96] {
    type C = *const [u8; 96];
    type Error = ();

    unsafe fn clone_from_repr_c(repr_c: Self::C) -> Result<Self, Self::Error> {
        Ok(*repr_c)
    }
}

impl ReprC for bool {
    type C = u32;
    type Error = ();

    unsafe fn clone_from_repr_c(repr_c: Self::C) -> Result<Self, Self::Error> {
        Ok(repr_c != 0)
    }
}
