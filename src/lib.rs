// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! FFI utilities.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/maidsafe/QA/master/Images/maidsafe_logo.png",
    html_favicon_url = "http://maidsafe.net/img/favicon.ico",
    test(attr(forbid(warnings)))
)]
// For explanation of lint checks, run `rustc -W help`.
#![warn(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]
// This crate makes liberal use of unsafe code to work with FFI.
#![allow(unsafe_code)]

pub mod bindgen_utils;
pub mod callback;
#[cfg(feature = "java")]
pub mod java;
pub mod result;
pub mod string;
pub mod test_utils;

mod b64;
mod catch_unwind;
mod macros;
mod repr_c;
mod vec;

pub use self::b64::{base64_decode, base64_encode};
pub use self::catch_unwind::{catch_unwind_cb, catch_unwind_result};
pub use self::repr_c::ReprC;
pub use self::result::{FfiResult, NativeResult, FFI_RESULT_OK};
pub use self::string::StringError;
pub use self::vec::{vec_clone_from_raw_parts, vec_from_raw_parts, vec_into_raw_parts, SafePtr};

use std::os::raw::c_void;

/// Type that holds opaque user data handed into FFI functions.
#[derive(Clone, Copy)]
pub struct OpaqueCtx(pub *mut c_void);
unsafe impl Send for OpaqueCtx {}

impl Into<*mut c_void> for OpaqueCtx {
    fn into(self) -> *mut c_void {
        self.0
    }
}

/// Trait for types that can be converted to integer error code.
pub trait ErrorCode {
    /// Return the error code corresponding to this instance.
    fn error_code(&self) -> i32;
}
