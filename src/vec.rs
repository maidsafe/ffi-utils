// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use std::mem;
use std::ptr;
use std::slice;

/// Provides FFI-safe pointers, as opposed to raw `as_ptr()` in `Vec` and `String` which can return
/// values such as `0x01` that can cause segmentation faults with the automatic pointer
/// dereferencing on the front-end side (e.g. in Node.js).
pub trait SafePtr {
    /// Resulting pointer type.
    type Ptr;

    /// Returns a pointer that guarantees safe dereferencing on the front-end side.
    fn as_safe_ptr(&self) -> *const Self::Ptr;
}

impl<T> SafePtr for Vec<T> {
    type Ptr = T;

    fn as_safe_ptr(&self) -> *const T {
        if self.is_empty() {
            ptr::null()
        } else {
            self.as_ptr()
        }
    }
}

/// Consumes a `Vec` and transfers ownership of the data to a C caller, returning (pointer, size).
///
/// The pointer which this function returns must be returned to Rust and reconstituted using
/// `vec_from_raw_parts` to be properly deallocated. Specifically, one should not use the standard C
/// `free()` function to deallocate this data.
///
/// Failure to call `vec_from_raw_parts` will lead to a memory leak.
pub fn vec_into_raw_parts<T>(v: Vec<T>) -> (*mut T, usize) {
    let mut b = v.into_boxed_slice();
    let ptr = b.as_mut_ptr();
    let len = b.len();
    mem::forget(b);
    (ptr, len)
}

/// Retakes ownership of a `Vec` that was transferred to C via `vec_into_raw_parts`.
pub unsafe fn vec_from_raw_parts<T>(ptr: *mut T, len: usize) -> Vec<T> {
    Box::from_raw(slice::from_raw_parts_mut(ptr, len)).into_vec()
}

/// Converts a pointer and length to `Vec` by cloning the contents.
/// Note: This does NOT free the memory pointed to by `ptr`.
pub unsafe fn vec_clone_from_raw_parts<T: Clone>(ptr: *const T, len: usize) -> Vec<T> {
    slice::from_raw_parts(ptr, len).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_conversions() {
        for _ in 0..5 {
            let v = vec!["foo", "bar"];

            for _ in 0..5 {
                let (ptr, len) = vec_into_raw_parts(v.clone());
                let v2 = unsafe { vec_clone_from_raw_parts(ptr, len) };
                assert_eq!(v, v2);
                let v3 = unsafe { vec_from_raw_parts(ptr, len) };
                assert_eq!(v, v3);
            }
        }
    }
}
