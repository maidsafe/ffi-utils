// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! Utilities for handling results and errors across the FFI boundary.

use crate::callback::{Callback, CallbackArgs};
use crate::string::{self, StringError};
use crate::{ErrorCode, ReprC};
use log::debug;
use std::ffi::CString;
use std::fmt::{Debug, Display};
use std::os::raw::{c_char, c_void};
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
                Some(string::from_c_str(description).map_err(StringError::from)?)
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

/// Convert a result into an `FfiResult` and call a callback.
pub fn call_result_cb<E, T, U>(result: Result<T, E>, user_data: U, cb: impl Callback)
where
    U: Into<*mut c_void>,
    E: Debug + Display + ErrorCode,
{
    let (error_code, description) = ffi_result(result);
    let res = unwrap!(NativeResult {
        error_code,
        description: Some(description),
    }
    .into_repr_c());
    cb.call(user_data.into(), &res, CallbackArgs::default());
}

/// Convert an error into a pair of `(error_code, description)` to be used in `FfiResult`.
pub fn ffi_error<E>(err: E) -> (i32, String)
where
    E: Debug + Display + ErrorCode,
{
    let err_code = ffi_error_code(&err);
    let err_desc = format!("{}", err);
    (err_code, err_desc)
}

/// Convert an error into an i32 error code.
pub fn ffi_error_code<E>(err: &E) -> i32
where
    E: Debug + ErrorCode,
{
    let err_str = format!("{:?}", err);
    let err_code = err.error_code();

    debug!("**ERRNO: {}** {}", err_code, err_str);
    err_code
}

/// Convert a result into a pair of `(error_code, description)` to be used in `FfiResult`.
pub fn ffi_result<E, T>(res: Result<T, E>) -> (i32, String)
where
    E: Debug + Display + ErrorCode,
{
    match res {
        Ok(_) => (0, String::default()),
        Err(error) => ffi_error(error),
    }
}

/// Convert a result into an i32 error code.
pub fn ffi_result_code<E, T>(res: Result<T, E>) -> i32
where
    E: Debug + ErrorCode,
{
    match res {
        Ok(_) => 0,
        Err(error) => ffi_error_code(&error),
    }
}
