// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::callback::{Callback, CallbackArgs};
use super::{ErrorCode, FfiResult, NativeResult};
use std::fmt::{Debug, Display};
use std::os::raw::c_void;
use std::panic::{self, AssertUnwindSafe};

/// Catches panics and returns the result.
pub fn catch_unwind_result<'a, F, T, E>(f: F) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
    E: Debug + From<&'a str>,
{
    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Err(_) => Err(E::from("panic")),
        Ok(result) => result,
    }
}

/// Catch panics. On error call the callback.
pub fn catch_unwind_cb<'a, U, C, F, E>(user_data: U, cb: C, f: F)
where
    U: Into<*mut c_void>,
    C: Callback + Copy,
    F: FnOnce() -> Result<(), E>,
    E: Debug + Display + ErrorCode + From<&'a str>,
{
    if let Err(err) = catch_unwind_result(f) {
        let (error_code, description) = ffi_result!(Err::<(), E>(err));
        let res = NativeResult {
            error_code,
            description: Some(description),
        }
        .into_repr_c();

        match res {
            Ok(res) => cb.call(user_data.into(), &res, CallbackArgs::default()),
            Err(_) => {
                let res = FfiResult {
                    error_code,
                    description: b"Could not convert error description into CString\x00"
                        as *const u8 as *const _,
                };
                cb.call(user_data.into(), &res, CallbackArgs::default());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestError;
    use crate::FfiResult;

    #[test]
    fn panic_inside_catch_unwind_result() {
        let mut did_unwind = false;

        let res = catch_unwind_result(|| -> Result<(), TestError> {
            let _probe = DropProbe::new(|| did_unwind = true);
            panic!("simulated panic");
        });

        assert!(res.is_err());
        assert!(did_unwind);
    }

    #[test]
    fn panic_inside_catch_unwind_cb() {
        extern "C" fn cb(user_data: *mut c_void, result: *const FfiResult) {
            unsafe {
                let error_code = user_data as *mut i32;
                *error_code = (*result).error_code;
            }
        }

        let mut did_unwind = false;
        let mut error_code = 0;

        let user_data: *mut i32 = &mut error_code;
        let user_data = user_data as *mut c_void;
        let cb: extern "C" fn(_, _) = cb;

        catch_unwind_cb(user_data, cb, || -> Result<(), TestError> {
            let _probe = DropProbe::new(|| did_unwind = true);
            panic!("simulated panic");
        });

        assert!(error_code < 0);
        assert!(did_unwind);
    }

    // Calls a callback on drop.
    struct DropProbe<F: FnOnce()>(Option<F>);

    impl<F: FnOnce()> DropProbe<F> {
        fn new(f: F) -> Self {
            DropProbe(Some(f))
        }
    }

    impl<F: FnOnce()> Drop for DropProbe<F> {
        fn drop(&mut self) {
            if let Some(f) = self.0.take() {
                f()
            }
        }
    }
}
