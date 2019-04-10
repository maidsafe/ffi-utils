// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! Test utilities.

use crate::repr_c::ReprC;
use crate::{ErrorCode, FfiResult};
use std::fmt::{Debug, Display};
use std::os::raw::c_void;
use std::sync::mpsc::{self, Sender};
use unwrap::unwrap;
use std::{fmt, ptr, slice};

/// User data wrapper.
pub struct UserData {
    /// Common field, used by standard callbacks.
    pub common: *mut c_void,
    /// Custom field, used by additional callbacks.
    pub custom: *mut c_void,
}

impl Default for UserData {
    fn default() -> Self {
        let common: *const c_void = ptr::null();
        let custom: *const c_void = ptr::null();

        UserData {
            common: common as *mut c_void,
            custom: custom as *mut c_void,
        }
    }
}

/// Convert a `UserData` to a void pointer which can be passed to ffi functions.
pub fn user_data_as_void(ud: &UserData) -> *mut c_void {
    let ptr: *const _ = ud;
    ptr as *mut c_void
}

/// Convert a `mpsc::Sender<T>` to a void ptr which is then stored in the `UserData` struct and
/// passed to ffi functions.
pub fn sender_as_user_data<T>(tx: &Sender<T>, ud: &mut UserData) -> *mut c_void {
    let ptr: *const _ = tx;
    ud.common = ptr as *mut c_void;
    user_data_as_void(ud)
}

/// Send through a `mpsc::Sender` pointed to by the user data's common pointer.
pub unsafe fn send_via_user_data<T>(user_data: *mut c_void, value: T)
where
    T: Send,
{
    let ud = user_data as *mut UserData;
    let tx = (*ud).common as *mut Sender<T>;
    unwrap!((*tx).send(value));
}

/// Send through a `mpsc::Sender` pointed to by the user data's custom pointer.
pub unsafe fn send_via_user_data_custom<T>(user_data: *mut c_void, value: T)
where
    T: Send,
{
    let ud = user_data as *mut UserData;
    let tx = (*ud).custom as *mut Sender<T>;
    unwrap!((*tx).send(value));
}

/// Call a FFI function and block until its callback gets called.
/// Use this if the callback accepts no arguments in addition to `user_data`
/// and `error_code`.
pub fn call_0<F>(f: F) -> Result<(), i32>
where
    F: FnOnce(*mut c_void, extern "C" fn(user_data: *mut c_void, result: *const FfiResult)),
{
    let mut ud = Default::default();
    call_0_with_custom(&mut ud, f)
}

/// Call a FFI function and block until its callback gets called.
/// Use this if the callback accepts no arguments in addition to `user_data`
/// and `error_code`.
/// This version of the function takes a `UserData` with custom inner data.
pub fn call_0_with_custom<F>(ud: &mut UserData, f: F) -> Result<(), i32>
where
    F: FnOnce(*mut c_void, extern "C" fn(user_data: *mut c_void, result: *const FfiResult)),
{
    let (tx, rx) = mpsc::channel::<i32>();
    f(sender_as_user_data(&tx, ud), callback_0);

    let error = unwrap!(rx.recv());
    if error == 0 {
        Ok(())
    } else {
        Err(error)
    }
}

/// Call an FFI function and block until its callback gets called, then return
/// the argument which were passed to that callback.
/// Use this if the callback accepts one argument in addition to `user_data`
/// and `error_code`.
pub unsafe fn call_1<F, E: Debug, T>(f: F) -> Result<T, i32>
where
    F: FnOnce(*mut c_void, extern "C" fn(user_data: *mut c_void, result: *const FfiResult, T::C)),
    T: ReprC<Error = E>,
{
    let mut ud = Default::default();
    call_1_with_custom(&mut ud, f)
}

/// Call an FFI function and block until its callback gets called, then return
/// the argument which were passed to that callback.
/// Use this if the callback accepts one argument in addition to `user_data`
/// and `error_code`.
/// This version of the function takes a `UserData` with custom inner data.
pub fn call_1_with_custom<F, E: Debug, T>(ud: &mut UserData, f: F) -> Result<T, i32>
where
    F: FnOnce(*mut c_void, extern "C" fn(user_data: *mut c_void, result: *const FfiResult, T::C)),
    T: ReprC<Error = E>,
{
    let (tx, rx) = mpsc::channel::<SendWrapper<Result<T, i32>>>();
    f(sender_as_user_data(&tx, ud), callback_1::<E, T>);
    unwrap!(rx.recv()).0
}

/// Call a FFI function and block until its callback gets called, then return
/// the argument which were passed to that callback.
/// Use this if the callback accepts two arguments in addition to `user_data`
/// and `error_code`.
pub unsafe fn call_2<F, E0, E1, T0, T1>(f: F) -> Result<(T0, T1), i32>
where
    F: FnOnce(
        *mut c_void,
        extern "C" fn(user_data: *mut c_void, result: *const FfiResult, T0::C, T1::C),
    ),
    E0: Debug,
    E1: Debug,
    T0: ReprC<Error = E0>,
    T1: ReprC<Error = E1>,
{
    let mut ud = Default::default();
    call_2_with_custom(&mut ud, f)
}

/// Call a FFI function and block until its callback gets called, then return
/// the argument which were passed to that callback.
/// Use this if the callback accepts two arguments in addition to `user_data`
/// and `error_code`.
/// This version of the function takes a `UserData` with custom inner data.
pub unsafe fn call_2_with_custom<F, E0, E1, T0, T1>(
    ud: &mut UserData,
    f: F,
) -> Result<(T0, T1), i32>
where
    F: FnOnce(
        *mut c_void,
        extern "C" fn(user_data: *mut c_void, result: *const FfiResult, T0::C, T1::C),
    ),
    E0: Debug,
    E1: Debug,
    T0: ReprC<Error = E0>,
    T1: ReprC<Error = E1>,
{
    let (tx, rx) = mpsc::channel::<SendWrapper<Result<(T0, T1), i32>>>();
    f(sender_as_user_data(&tx, ud), callback_2::<E0, E1, T0, T1>);
    unwrap!(rx.recv()).0
}

/// Call a FFI function and block until its callback gets called, then copy
/// the array argument which was passed to `Vec<T>` and return the result.
/// Use this if the callback accepts `*const T` and `usize` (length) arguments in addition
/// to `user_data` and `error_code`.
pub unsafe fn call_vec<F, E, T, U>(f: F) -> Result<Vec<T>, i32>
where
    F: FnOnce(
        *mut c_void,
        extern "C" fn(user_data: *mut c_void, result: *const FfiResult, T::C, usize),
    ),
    E: Debug,
    T: ReprC<C = *const U, Error = E>,
{
    let mut ud = Default::default();
    call_vec_with_custom(&mut ud, f)
}

/// Call a FFI function and block until its callback gets called, then copy
/// the array argument which was passed to `Vec<T>` and return the result.
/// Use this if the callback accepts `*const T` and `usize` (length) arguments in addition
/// to `user_data` and `error_code`.
/// This version of the function takes a `UserData` with custom inner data.
pub unsafe fn call_vec_with_custom<F, E, T, U>(ud: &mut UserData, f: F) -> Result<Vec<T>, i32>
where
    F: FnOnce(
        *mut c_void,
        extern "C" fn(user_data: *mut c_void, result: *const FfiResult, T::C, usize),
    ),
    E: Debug,
    T: ReprC<C = *const U, Error = E>,
{
    let (tx, rx) = mpsc::channel::<SendWrapper<Result<Vec<T>, i32>>>();
    f(sender_as_user_data(&tx, ud), callback_vec::<E, T, U>);
    unwrap!(rx.recv()).0
}

/// Call a FFI function and block until its callback gets called, then copy
/// the byte array argument which was passed to `Vec<u8>` and return the result.
pub unsafe fn call_vec_u8<F>(f: F) -> Result<Vec<u8>, i32>
where
    F: FnOnce(
        *mut c_void,
        extern "C" fn(user_data: *mut c_void, result: *const FfiResult, *const u8, usize),
    ),
{
    let mut ud = Default::default();
    call_vec_u8_with_custom(&mut ud, f)
}

/// Call a FFI function and block until its callback gets called, then copy
/// the byte array argument which was passed to `Vec<u8>` and return the result.
/// This version of the function takes a `UserData` with custom inner data.
/// This version of the function takes a `UserData` with custom inner data.
pub unsafe fn call_vec_u8_with_custom<F>(ud: &mut UserData, f: F) -> Result<Vec<u8>, i32>
where
    F: FnOnce(
        *mut c_void,
        extern "C" fn(user_data: *mut c_void, result: *const FfiResult, *const u8, usize),
    ),
{
    let (tx, rx) = mpsc::channel::<Result<Vec<u8>, i32>>();
    f(sender_as_user_data(&tx, ud), callback_vec_u8);
    unwrap!(rx.recv())
}

extern "C" fn callback_0(user_data: *mut c_void, res: *const FfiResult) {
    unsafe { send_via_user_data(user_data, (*res).error_code) }
}

extern "C" fn callback_1<E, T>(user_data: *mut c_void, res: *const FfiResult, arg: T::C)
where
    E: Debug,
    T: ReprC<Error = E>,
{
    unsafe {
        let result: Result<T, i32> = if (*res).error_code == 0 {
            Ok(unwrap!(T::clone_from_repr_c(arg)))
        } else {
            Err((*res).error_code)
        };
        send_via_user_data(user_data, SendWrapper(result));
    }
}

extern "C" fn callback_2<E0, E1, T0, T1>(
    user_data: *mut c_void,
    res: *const FfiResult,
    arg0: T0::C,
    arg1: T1::C,
) where
    E0: Debug,
    E1: Debug,
    T0: ReprC<Error = E0>,
    T1: ReprC<Error = E1>,
{
    unsafe {
        let result: Result<(T0, T1), i32> = if (*res).error_code == 0 {
            Ok((
                unwrap!(T0::clone_from_repr_c(arg0)),
                unwrap!(T1::clone_from_repr_c(arg1)),
            ))
        } else {
            Err((*res).error_code)
        };
        send_via_user_data(user_data, SendWrapper(result))
    }
}

extern "C" fn callback_vec<E, T, U>(
    user_data: *mut c_void,
    res: *const FfiResult,
    array: *const U,
    size: usize,
) where
    E: Debug,
    T: ReprC<C = *const U, Error = E>,
{
    unsafe {
        let result: Result<Vec<T>, i32> = if (*res).error_code == 0 {
            let slice_ffi = slice::from_raw_parts(array, size);
            let mut vec = Vec::with_capacity(slice_ffi.len());
            for elt in slice_ffi {
                vec.push(unwrap!(T::clone_from_repr_c(elt)));
            }
            Ok(vec)
        } else {
            Err((*res).error_code)
        };

        send_via_user_data(user_data, SendWrapper(result))
    }
}

extern "C" fn callback_vec_u8(
    user_data: *mut c_void,
    res: *const FfiResult,
    ptr: *const u8,
    len: usize,
) {
    unsafe {
        let result = if (*res).error_code == 0 {
            Ok(slice::from_raw_parts(ptr, len).to_vec())
        } else {
            Err((*res).error_code)
        };

        send_via_user_data(user_data, result)
    }
}

/// Unsafe wrapper for passing non-Send types through mpsc channels.
/// Use with caution!
pub struct SendWrapper<T>(pub T);
unsafe impl<T> Send for SendWrapper<T> {}

/// Dummy error type for testing that implements ErrorCode.
#[derive(Debug)]
pub enum TestError {
    /// Error from a string.
    FromStr(String),
    /// Simple test error.
    Test,
}

impl<'a> From<&'a str> for TestError {
    fn from(s: &'a str) -> Self {
        TestError::FromStr(s.into())
    }
}

impl ErrorCode for TestError {
    fn error_code(&self) -> i32 {
        use TestError::*;
        match *self {
            Test => -1,
            FromStr(_) => -2,
        }
    }
}

impl Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TestError::*;
        match self {
            Test => write!(f, "Test Error"),
            FromStr(s) => write!(f, "{}", s),
        }
    }
}
