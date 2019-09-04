// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! FFI macros.
//!
//! Because the `debug!` log output provides helpful information about FFI errors, these macros
//! cannot be functions. Otherwise we lose some debug data like the line and column numbers and
//! module name.

/// Convert an error into a pair of `(error_code: i32, description: String)` to be used in
/// `NativeResult`.
#[macro_export]
macro_rules! ffi_error {
    ($err:expr) => {{
        let err_code = ffi_error_code!($err);
        let err_desc = format!("{}", $err);
        (err_code, err_desc)
    }};
}

/// Convert a result into a pair of `(error_code: i32, description: String)` to be used in
/// `NativeResult`.
#[macro_export]
macro_rules! ffi_result {
    ($res:expr) => {
        match $res {
            Ok(_) => (0, String::default()),
            Err(error) => ffi_error!(error),
        }
    };
}

/// Convert a result into an i32 error code.
#[macro_export]
macro_rules! ffi_result_code {
    ($res:expr) => {
        match $res {
            Ok(_) => 0,
            Err(error) => ffi_error_code!(error),
        }
    };
}

/// Convert an error into an i32 error code.
#[macro_export]
macro_rules! ffi_error_code {
    ($err:expr) => {{
        #[allow(unused_imports, clippy::useless_attribute)]
        use $crate::ErrorCode;

        let err = &$err;
        let err_str = format!("{:?}", err);
        let err_code = err.error_code();

        log::debug!("**ERRNO: {}** {}", err_code, err_str);
        err_code
    }};
}

/// Convert a result into an `FfiResult` and call a callback.
#[macro_export]
macro_rules! call_result_cb {
    ($result:expr, $user_data:expr, $cb:expr) => {
        #[allow(unused_imports, clippy::useless_attribute)]
        use $crate::callback::{Callback, CallbackArgs};
        use $crate::result::{FfiResult, NativeResult};

        let (error_code, description) = ffi_result!($result);
        let res = NativeResult {
            error_code,
            description: Some(description),
        }
        .into_repr_c();

        match res {
            Ok(res) => $cb.call($user_data.into(), &res, CallbackArgs::default()),
            Err(_) => {
                let res = FfiResult {
                    error_code,
                    description: b"Could not convert error description into CString\x00"
                        as *const u8 as *const _,
                };
                $cb.call($user_data.into(), &res, CallbackArgs::default());
            }
        }
    };
}

/// Given a result, calls the callback if it is an error, otherwise produces the wrapped value.
/// Should be called within `catch_unwind`, so returns `None` on error.
#[macro_export]
macro_rules! try_cb {
    ($result:expr, $user_data:expr, $cb:expr) => {
        match $result {
            Ok(value) => value,
            e @ Err(_) => {
                call_result_cb!(e, $user_data, $cb);
                return None;
            }
        }
    };
}
