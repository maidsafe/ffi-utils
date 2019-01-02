// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

/// Converts a result into a pair of `(error_code: i32, description: CString)`
/// to be used in `FfiResult`
#[macro_export]
macro_rules! ffi_error {
    ($error:expr) => {{
        let err_code = ffi_error_code!($error);
        let err_desc = format!("{}", $error);
        (err_code, unwrap!(::std::ffi::CString::new(err_desc)))
    }};
}

/// Converts a result into a pair of `(error_code: i32, description: CString)`
/// to be used in `FfiResult`
#[macro_export]
macro_rules! ffi_result {
    ($res:expr) => {
        match $res {
            Ok(_) => (0, ::std::ffi::CString::default()),
            Err(error) => ffi_error!(error),
        }
    };
}

#[macro_export]
macro_rules! ffi_result_code {
    ($res:expr) => {
        match $res {
            Ok(_) => 0,
            Err(error) => ffi_error_code!(error),
        }
    };
}

#[macro_export]
macro_rules! ffi_error_code {
    ($err:expr) => {{
        #[allow(unused, clippy::useless_attribute)]
        use $crate::ErrorCode;

        let err = &$err;
        let err_str = format!("{:?}", err);
        let err_code = err.error_code();

        debug!("**ERRNO: {}** {}", err_code, err_str);
        err_code
    }};
}

#[macro_export]
macro_rules! call_result_cb {
    ($result:expr, $user_data:expr, $cb:expr) => {
        #[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
        #[allow(unused)]
        use $crate::callback::{Callback, CallbackArgs};
        let (error_code, description) = ffi_result!($result);
        let res = FfiResult {
            error_code,
            description: description.as_ptr(),
        };
        $cb.call($user_data.into(), &res, CallbackArgs::default());
    };
}

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
