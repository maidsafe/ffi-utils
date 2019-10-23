// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! Integration tests for FFI utilities.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/maidsafe/QA/master/Images/maidsafe_logo.png",
    html_favicon_url = "http://maidsafe.net/img/favicon.ico",
    test(attr(forbid(warnings)))
)]
// For explanation of lint checks, run `rustc -W help` or see
// https://github.com/maidsafe/QA/blob/master/Documentation/Rust%20Lint%20Checks.md
#![forbid(
    exceeding_bitshifts,
    mutable_transmutes,
    no_mangle_const_items,
    unknown_crate_types,
    warnings
)]
#![deny(
    bad_style,
    deprecated,
    improper_ctypes,
    missing_docs,
    non_shorthand_field_patterns,
    overflowing_literals,
    plugin_as_library,
    stable_features,
    unconditional_recursion,
    unknown_lints,
    unused,
    unused_allocation,
    unused_attributes,
    unused_comparisons,
    unused_features,
    unused_parens,
    while_true,
    clippy::all,
    clippy::unicode_not_nfc,
    clippy::wrong_pub_self_convention,
    clippy::option_unwrap_used
)]
#![warn(
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]
#![allow(
    box_pointers,
    missing_copy_implementations,
    missing_debug_implementations,
    variant_size_differences,
    clippy::implicit_hasher,
    clippy::too_many_arguments,
    clippy::use_debug
)]

#[macro_use]
extern crate ffi_utils;
#[macro_use]
extern crate unwrap;

// Test the basic example from our "FFI calling conventions" doc.
#[test]
fn basic() {
    use ffi_utils::test_utils::TestError;
    use ffi_utils::{catch_unwind_cb, FfiResult, OpaqueCtx, FFI_RESULT_OK};
    use std::os::raw::c_void;

    // A typical FFI function. Returns `input_param * 42`.
    #[no_mangle]
    unsafe extern "C" fn foreign_function(
        input_param: i32,
        user_data: *mut c_void,
        o_callback: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, value: i32),
    ) {
        let user_data = OpaqueCtx(user_data);

        catch_unwind_cb(user_data, o_callback, || -> Result<_, TestError> {
            // Induce a panic on overflow in both debug and release builds.
            let (output, overflow) = input_param.overflowing_mul(42);
            if overflow {
                panic!();
            }

            o_callback(user_data.0, FFI_RESULT_OK, output);

            Ok(())
        })
    }

    // Test the example.
    {
        use ffi_utils::test_utils::call_1;

        // Test success case.
        let val: i32 = unsafe { unwrap!(call_1(|ud, cb| foreign_function(1, ud, cb))) };
        assert_eq!(val, 42);

        // Test catching a panic.
        let res: Result<i32, i32> =
            unsafe { call_1(|ud, cb| foreign_function(::std::i32::MAX, ud, cb)) };
        match res {
            Ok(value) => panic!("Unexpected value: {:?}", value),
            Err(-2) => (),
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}

// Test the utility functions as covered in "FFI calling conventions".
#[test]
fn utility_functions() {
    use ffi_utils::call_result_cb;
    use ffi_utils::test_utils::TestError;
    use ffi_utils::{catch_unwind_cb, FfiResult, OpaqueCtx, FFI_RESULT_OK};
    use std::os::raw::c_void;

    // Function that returns a Result.
    fn multiply_by_42(input_param: i32) -> Result<i32, TestError> {
        let (output, overflow) = input_param.overflowing_mul(42);
        if overflow {
            Err(TestError::FromStr("Overflow detected and prevented".into()))
        } else {
            Ok(output)
        }
    }

    // A typical FFI function. Returns `input_param * 42`.
    #[no_mangle]
    unsafe extern "C" fn foreign_function2(
        input_param: i32,
        user_data: *mut c_void,
        o_callback: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, value: i32),
    ) {
        let user_data = OpaqueCtx(user_data);

        catch_unwind_cb(user_data, o_callback, || -> Result<_, TestError> {
            match multiply_by_42(input_param) {
                Ok(output) => o_callback(user_data.0, FFI_RESULT_OK, output),
                Err(e) => {
                    call_result_cb!(Err::<(), _>(e), user_data, o_callback);
                }
            }

            Ok(())
        })
    }

    // Test the example.
    {
        use ffi_utils::NativeResult;
        use utils::call_1_ffi_result;

        // Test success case.
        let val: i32 = unsafe { unwrap!(call_1_ffi_result(|ud, cb| foreign_function2(1, ud, cb))) };
        assert_eq!(val, 42);

        // Test error case.
        let res: Result<i32, NativeResult> =
            unsafe { call_1_ffi_result(|ud, cb| foreign_function2(::std::i32::MAX, ud, cb)) };
        match res {
            Ok(_) => panic!("Unexpected value"),
            Err(native_result) => {
                assert_eq!(native_result.error_code, -2);
                assert_eq!(
                    native_result.description,
                    Some("Overflow detected and prevented".into())
                );
            }
        }
    }
}

mod utils {
    use ffi_utils::test_utils::{send_via_user_data, sender_as_user_data, SendWrapper};
    use ffi_utils::{FfiResult, NativeResult, ReprC};
    use std::fmt::Debug;
    use std::os::raw::c_void;
    use std::sync::mpsc;

    pub unsafe fn call_1_ffi_result<F, E: Debug, T>(f: F) -> Result<T, NativeResult>
    where
        F: FnOnce(
            *mut c_void,
            extern "C" fn(user_data: *mut c_void, result: *const FfiResult, T::C),
        ),
        T: ReprC<Error = E>,
    {
        let (tx, rx) = mpsc::channel::<SendWrapper<Result<T, NativeResult>>>();
        f(
            sender_as_user_data(&tx, &mut Default::default()),
            callback_1_ffi_result::<E, T>,
        );
        unwrap!(rx.recv()).0
    }

    extern "C" fn callback_1_ffi_result<E, T>(
        user_data: *mut c_void,
        res: *const FfiResult,
        arg: T::C,
    ) where
        E: Debug,
        T: ReprC<Error = E>,
    {
        unsafe {
            let result: Result<T, NativeResult> = if (*res).error_code == 0 {
                Ok(unwrap!(T::clone_from_repr_c(arg)))
            } else {
                Err(unwrap!(NativeResult::clone_from_repr_c(res)))
            };
            send_via_user_data(user_data, SendWrapper(result));
        }
    }
}
