// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! Helpers to work with extern "C" callbacks.

use crate::result::FfiResult;
use std::os::raw::c_void;
use std::ptr;

/// Given a result, calls the callback if it is an error, otherwise produces the wrapped value.
/// Should be called within `catch_unwind`, so returns `None` on error.
#[macro_export]
macro_rules! try_cb {
    ($result:expr, $user_data:expr, $cb:expr) => {
        match $result {
            Ok(value) => value,
            e @ Err(_) => {
                result::call_result_cb(e, $user_data, $cb);
                return None;
            }
        }
    };
}

/// This trait allows us to treat callbacks with different number and type of arguments uniformly.
pub trait Callback {
    /// Arguments for the callback. Should be a tuple.
    type Args: CallbackArgs;

    /// Call the callback, passing the user data pointer, error code and any additional arguments.
    fn call(&self, user_data: *mut c_void, error: *const FfiResult, args: Self::Args);
}

impl Callback for extern "C" fn(user_data: *mut c_void, result: *const FfiResult) {
    type Args = ();
    fn call(&self, user_data: *mut c_void, error: *const FfiResult, _args: Self::Args) {
        self(user_data, error)
    }
}

impl<T: CallbackArgs> Callback
    for extern "C" fn(user_data: *mut c_void, result: *const FfiResult, a: T)
{
    type Args = T;
    fn call(&self, user_data: *mut c_void, error: *const FfiResult, args: Self::Args) {
        self(user_data, error, args)
    }
}

impl<T: CallbackArgs> Callback
    for unsafe extern "C" fn(user_data: *mut c_void, result: *const FfiResult, a: T)
{
    type Args = T;
    fn call(&self, user_data: *mut c_void, error: *const FfiResult, args: Self::Args) {
        unsafe { self(user_data, error, args) }
    }
}

impl<T0: CallbackArgs, T1: CallbackArgs> Callback
    for extern "C" fn(user_data: *mut c_void, result: *const FfiResult, a0: T0, a1: T1)
{
    type Args = (T0, T1);
    fn call(&self, user_data: *mut c_void, error: *const FfiResult, args: Self::Args) {
        self(user_data, error, args.0, args.1)
    }
}

impl<T0: CallbackArgs, T1: CallbackArgs, T2: CallbackArgs> Callback
    for extern "C" fn(user_data: *mut c_void, result: *const FfiResult, a0: T0, a1: T1, a2: T2)
{
    type Args = (T0, T1, T2);
    fn call(&self, user_data: *mut c_void, error: *const FfiResult, args: Self::Args) {
        self(user_data, error, args.0, args.1, args.2)
    }
}

/// Trait for arguments to callbacks. This is similar to `Default`, but allows
/// us to implement it for foreign types that don't already implement `Default`.
pub trait CallbackArgs {
    /// Return default value for the type, used when calling the callback with error.
    fn default() -> Self;
}

impl CallbackArgs for () {
    fn default() -> Self {}
}

impl CallbackArgs for bool {
    fn default() -> Self {
        false
    }
}

impl CallbackArgs for u32 {
    fn default() -> Self {
        0
    }
}

impl CallbackArgs for i32 {
    fn default() -> Self {
        0
    }
}

impl CallbackArgs for i64 {
    fn default() -> Self {
        0
    }
}

impl CallbackArgs for u64 {
    fn default() -> Self {
        0
    }
}

impl CallbackArgs for usize {
    fn default() -> Self {
        0
    }
}

impl<T> CallbackArgs for *const T {
    fn default() -> Self {
        ptr::null()
    }
}

impl<T> CallbackArgs for *mut T {
    fn default() -> Self {
        ptr::null_mut()
    }
}

impl CallbackArgs for [u8; 32] {
    fn default() -> Self {
        [0; 32]
    }
}

impl<T0: CallbackArgs, T1: CallbackArgs> CallbackArgs for (T0, T1) {
    fn default() -> Self {
        (CallbackArgs::default(), CallbackArgs::default())
    }
}

impl<T0: CallbackArgs, T1: CallbackArgs, T2: CallbackArgs> CallbackArgs for (T0, T1, T2) {
    fn default() -> Self {
        (
            CallbackArgs::default(),
            CallbackArgs::default(),
            CallbackArgs::default(),
        )
    }
}

impl<T0: CallbackArgs, T1: CallbackArgs, T2: CallbackArgs, T3: CallbackArgs> CallbackArgs
    for (T0, T1, T2, T3)
{
    fn default() -> Self {
        (
            CallbackArgs::default(),
            CallbackArgs::default(),
            CallbackArgs::default(),
            CallbackArgs::default(),
        )
    }
}
