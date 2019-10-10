# FFI utils - Change Log

## [0.13.0]
- Make `vec_into_raw_parts` not return a capacity
- Add `vec_from_raw_parts`
- Remove `from_c_str`

## [0.12.0]
- Use stable rustc, clippy, and rustfmt
- Upgrade crate to Rust 2018 edition
- Fix compiler and clippy errors
- Update jni to 0.12.0
- Introduce `NativeResult` type
- Change macros to interface with `NativeResult`
- Move `TestError` to test_utils.rs
- Add `ReprC` implementation for `i32` and `i64` types

## [0.11.0]
- Fix leaking local references in the Java module. Because the Android local reference table is limited to 512
  entries it is important to deallocate the local references as soon as possible.

## [0.10.0]
- Make the `java` module feature-gated. This should optimise build times for dependent crates which don't require Java/JNI support
- Add a new helper enum `EnvGuard` that simplifies JNI env management (acquiring, holding the reference, and deallocating it automatically)
- Make JNI code safer and cleaner

## [0.9.0]
- Support providing custom class loader in Java/JNI routines and macros

## [0.8.0]
- Update to dual license (MIT/BSD)
- Upgrade unwrap version to 1.2.0
- Use rust 1.28.0 stable / 2018-07-07 nightly
- rustfmt 0.99.2 and clippy-0.0.212

## [0.7.0]
- Add a public helper function `catch_unwind_result` for synchronous APIs (in addition to `catch_unwind_cb` for async)

## [0.6.0]
- Use rust 1.26.1 stable / 2018-02-29 nightly
- rustfmt-nightly 0.8.2 and clippy-0.0.206
- Updated license from dual Maidsafe/GPLv3 to GPLv3
- Add binding generator utilities

## [0.5.0]
- Use rust 1.22.1 stable / 2018-01-10 nightly
- rustfmt 0.9.0 and clippy-0.0.179
- `catch_unwind_error_code` function removed as it was no longer used

## [0.4.0]
- Use pointers to `FfiResult` instead of passing by value
- Change type of `FFI_RESULT_OK` to a static reference
- Don't add padding to URIs
- Update base64 version
- Add support for using a single user data parameter for multiple callbacks
- Add tests for the `catch_unwind` family of functions

## [0.3.0]
- Improve documentation and fix bugs
- Fix compiler errors on rustc-nightly

## [0.2.0]
- Change the log output for FFI errors - remove the decoration and reduce the log level

## [0.1.0]
- Provide FFI utility functions for safe_client_libs
