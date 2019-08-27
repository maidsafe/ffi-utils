// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! Utilities for handling results and errors across the FFI boundary.

use crate::string::StringError;
use crate::ReprC;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

/// Constant value to be used for OK result.
pub const FFI_RESULT_OK: &FfiResult = &FfiResult {
    error_code: 0,
    description: ptr::null(),
};

/// A native Rust version of the `FfiResult` struct.
#[derive(Clone, Debug)]
pub struct NativeResult {
    /// Unique error code.
    pub error_code: i32,
    /// Error description.
    pub description: Option<String>,
}

impl NativeResult {
    /// Construct FFI wrapper for the native Rust object, consuming self.
    pub fn into_repr_c(self) -> Result<FfiResult, StringError> {
        Ok(FfiResult {
            error_code: self.error_code,
            description: match self.description {
                Some(description) => CString::new(description)
                    .map_err(StringError::from)?
                    .into_raw(),
                None => ptr::null(),
            },
        })
    }
}

impl ReprC for NativeResult {
    type C = *const FfiResult;
    type Error = StringError;

    unsafe fn clone_from_repr_c(repr_c: Self::C) -> Result<Self, Self::Error> {
        let FfiResult {
            error_code,
            description,
        } = *repr_c;

        Ok(Self {
            error_code,
            description: if description.is_null() {
                None
            } else {
                Some(String::clone_from_repr_c(description).map_err(StringError::from)?)
            },
        })
    }
}

/// FFI result wrapper.
#[repr(C)]
#[derive(Debug)]
pub struct FfiResult {
    /// Unique error code.
    pub error_code: i32,
    /// Error description.
    pub description: *const c_char,
}

impl Drop for FfiResult {
    fn drop(&mut self) {
        unsafe {
            if !self.description.is_null() {
                let _ = CString::from_raw(self.description as *mut _);
            }
        }
    }
}
